use futures::TryStreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use r2d2_redis::RedisConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::{sync::Mutex, task};
use tokio_stream::StreamExt;
use diesel::{r2d2::ConnectionManager, ExpressionMethods, RunQueryDsl};
use crate::db::cluster_helper::{create_execute_ssh_script, get_available_compute_unit, release_compute_unit_lock, update_cluster_state};
use crate::models::schema::Compute_Unit;
use diesel::PgConnection;
use diesel::r2d2::Pool;
use diesel::query_dsl::methods::FilterDsl;

pub type RabbitMQPool = Arc<Mutex<Connection>>;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Serialize)]
struct ClusterConfig {
    name: String,
    cpu_limit: u32,
    ram_limit: u32,
    disk_size: String,
}

pub async fn create_rabbitmq_pool(rabbitmq_uri: String) -> RabbitMQPool {
    let connection = Connection::connect(&rabbitmq_uri, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");
    Arc::new(Mutex::new(connection))
}


pub async fn start_rabbitmq_cluster_message_consumer(
    rabbitmq_pool: Arc<Mutex<lapin::Connection>>,
    db_pool: DbPool,
    cache_pool: Pool<RedisConnectionManager>,
    queue_name: String,
) {
    let connection = rabbitmq_pool.lock().await;
    let channel = match connection.create_channel().await {
        Ok(channel) => channel,
        Err(err) => {
            eprintln!("❌ Failed to create RabbitMQ channel: {:?}", err);
            return;
        }
    };

    // ✅ Declare main queue
    if let Err(err) = channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                exclusive: false,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
    {
        eprintln!("❌ Failed to declare queue '{}': {:?}", queue_name, err);
        return;
    };

    // ✅ Declare retry queue
    let retry_queue = format!("{}_retry", queue_name);
    let mut retry_args = FieldTable::default();
    retry_args.insert("x-message-ttl".into(), AMQPValue::LongUInt(30_000)); // 30s delay
    retry_args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("".to_string().into()));
    retry_args.insert("x-dead-letter-routing-key".into(), AMQPValue::LongString(queue_name.clone().into()));

    if let Err(err) = channel
        .queue_declare(
            &retry_queue,
            QueueDeclareOptions {
                durable: true,
                exclusive: false,
                auto_delete: false,
                ..Default::default()
            },
            retry_args,
        )
        .await
    {
        eprintln!("❌ Failed to declare retry queue '{}': {:?}", retry_queue, err);
        return;
    }

    // ✅ Start consumer
    let consumer: Consumer = match channel
        .basic_consume(
            &queue_name,
            "consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(consumer) => consumer,
        Err(err) => {
            eprintln!("❌ Failed to start consumer: {:?}", err);
            return;
        }
    };

    println!("✅ Consumer started on queue '{}'", queue_name);

    let channel_clone = channel.clone();

    tokio::spawn(async move {
        let mut consumer_stream = consumer.into_stream();

        
        while let Some(delivery_result) = futures::TryStreamExt::try_next(&mut consumer_stream).await.unwrap_or(None) {
            let channel_clone = channel_clone.clone();
            let db_pool = db_pool.clone();
            let cache_pool = cache_pool.clone();
            let retry_queue = retry_queue.clone();

            tokio::spawn(async move {
                match handle_create_cluster_message(delivery_result, &channel_clone, &db_pool, &cache_pool, &retry_queue).await {
                    Ok(_) => {}
                    Err(err) => eprintln!("❌ Error processing message: {:?}", err),
                }
            });
        }
    });
}

async fn handle_create_cluster_message(
    delivery: lapin::message::Delivery,
    channel: &Channel,
    db_pool: &DbPool,
    cache_pool: &Pool<RedisConnectionManager>,
    retry_queue: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = String::from_utf8_lossy(&delivery.data);
    println!("📩 Received message: {}", message);

    let parsed_message: Result<Value, _> = serde_json::from_str(&message);
    if let Ok(json_data) = parsed_message {
        let cluster_name = json_data["name"].as_str().unwrap_or("default-cluster");
        let cpus = json_data["cpu_limit"].as_f64().map(|v| v.round() as i64).unwrap_or(2);
        let memory = format!("{}g", json_data["ram_limit"].as_f64().unwrap_or(2.0));
        let disk_size = format!("{}g", json_data["disk_size"].as_i64().unwrap_or(10));

        let mut conn = db_pool.get().expect("Failed to get DB connection");
        let mut cache_conn = cache_pool.get().expect("Failed to get cache connection");

        match get_available_compute_unit(&mut conn, &mut cache_conn, "ap_south_1") {
            Ok(Some(unit)) => {
                let ssh_host = &unit.fqdn;
                let ssh_user = "dc0102";
                let script_path = "/home/dc0102/Documents/rackmint-infra-as-code/create-cluster.sh";

                if let Err(err) = update_cluster_state(db_pool, cluster_name, "init").await {
                    eprintln!("❌ Failed to update cluster '{}' state: {:?}", cluster_name, err);
                    return Err(err.into());
                }
                println!("Executing on {:?}" , unit);
                sleep(Duration::from_secs(10)).await;

                
                match create_execute_ssh_script(ssh_host, ssh_user, script_path, cluster_name, cpus, &memory, &disk_size).await {
                    Ok(_) => {
                        println!("🎉 Cluster creation completed successfully!");
                        release_compute_unit_lock(&mut cache_conn, unit.id);
                        update_cluster_state(db_pool, cluster_name, "running").await?;
                    }
                    Err(error) => eprintln!("❌ Cluster creation failed: {}", error),
                }
            }
            Ok(None) => {
                println!("⚠️ No available compute unit. Retrying message...");
                channel
                    .basic_publish(
                        "",
                        retry_queue,
                        BasicPublishOptions::default(),
                        &delivery.data,
                        BasicProperties::default(),
                    )
                    .await?;
            }
            Err(err) => {
                println!("❌ Error querying compute units: {:?}", err);
            }
        }
    } else {
        eprintln!("❌ Invalid message format: {}", message);
    }

    // ✅ Acknowledge the message
    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}
