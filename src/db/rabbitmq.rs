use futures::TryStreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions}, types::FieldTable, Connection, ConnectionProperties, Consumer,
};
use r2d2_redis::RedisConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::{sync::Mutex, task};
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};
use rocket::State;
use diesel::{r2d2::ConnectionManager, ExpressionMethods, RunQueryDsl};
use crate::{db::cluster_helper::{create_execute_ssh_script, get_available_compute_unit, release_compute_unit_lock}, models::schema::Compute_Unit}; // ✅ Required for `.next()`
use diesel::PgConnection;
use diesel::r2d2::Pool;
pub type RabbitMQPool = Arc<Mutex<Connection>>;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
use diesel::query_dsl::methods::FilterDsl;
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

// ✅ Update cluster state in the database
pub async fn update_cluster_state(db_pool: &DbPool, cluster_name: &str, new_state: &str) -> Result<(), diesel::result::Error> {
    let mut conn = db_pool.get().expect("Failed to get DB connection");
    use crate::models::schema::schema::cluster::dsl::*;


    diesel::update(cluster.filter(name.eq(cluster_name)))
        .set(state.eq(new_state))
        .execute(&mut conn)?;

    println!("✅ Cluster '{}' state updated to '{}'", cluster_name, new_state);
    Ok(())
}


pub async fn start_rabbitmq_consumer(
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

    // ✅ Ensure queue exists before consuming
    match channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true, // Queue persists even after RabbitMQ restarts
                exclusive: false,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
    {
        Ok(_) => println!("✅ Queue '{}' is ready", queue_name),
        Err(err) => {
            eprintln!("❌ Failed to declare queue '{}': {:?}", queue_name, err);
            return;
        }
    };

    // ✅ Now start the consumer
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
// ✅ Clone db_pool so it can be moved into the async task
    let db_pool = db_pool.clone(); 
    
    task::spawn(async move {
        let mut consumer_stream = consumer.into_stream();

        while let Some(delivery_result) = consumer_stream.next().await {
            match delivery_result {
                Ok(delivery) => {
                    let message = String::from_utf8_lossy(&delivery.data);
                    println!("📩 Received message: {}", message);

                    let parsed_message: Result<Value, _> = serde_json::from_str(&message);
                    if let Ok(json_data) = parsed_message {
                        let cluster_name = json_data["name"].as_str().unwrap_or("default-cluster");
                        let cpus = json_data["cpu_limit"]
                            .as_f64()
                            .map(|v| v.round() as i64)  // Round to nearest integer
                            .unwrap_or(2);  // Default to 2 if missing

                        let memory = format!("{}g", json_data["ram_limit"].as_f64().unwrap_or(2.0));  // Defaults to 2.0
                        let disk_size = format!("{}g", json_data["disk_size"].as_i64().unwrap_or(10)); // Defaults to 10

                        let mut conn = db_pool.get().expect("Failed to get DB connection");
                        let mut cache_conn = cache_pool.get().expect("Failed to get cache connection");

                                                // Get an available compute unit that is not locked
                        let cu: Option<Compute_Unit> = match get_available_compute_unit(&mut conn, &mut cache_conn, "ap_south_1") {
                            Ok(Some(unit)) => Some(unit),
                            Ok(None) => {
                                println!("No available compute unit found in the requested region");
                                None
                            }
                            Err(_) => {
                                println!("Error querying compute units");
                                None
                            }
                        };

                        // proceed only if there is a CU available



                        let ssh_host = &cu.clone().unwrap().fqdn; // ✅ Change this to your target machine
                        let ssh_user = "dc0102"; // ✅ SSH username
                        let script_path = "/home/dc0102/Documents/rackmint-infra-as-code/create-cluster.sh"; // ✅ Remote script path


                        // ✅ Update cluster state to "init"
                        if let Err(err) = update_cluster_state(&db_pool, cluster_name, "init").await {
                            eprintln!("❌ Failed to update cluster '{}' state: {:?}", cluster_name, err);
                            continue;
                        }

                        match create_execute_ssh_script(ssh_host, ssh_user, script_path, cluster_name, cpus, &memory, &disk_size).await {
                            Ok(_) => {
                                println!("🎉 Cluster creation completed successfully!");
                                // ✅ Release lock after cluster creation
                                release_compute_unit_lock(&mut cache_conn, cu.unwrap().id);
                                if let Err(err) = update_cluster_state(&db_pool, cluster_name, "running").await {
                                    eprintln!("❌ Failed to update cluster '{}' state to 'running': {:?}", cluster_name, err);
                                }
                            },
                            Err(error) => eprintln!("❌ Cluster creation failed: {}", error),
                        }
                    } else {
                        eprintln!("❌ Invalid message format: {}", message);
                    }
                    

                    // ✅ Acknowledge the message
                    if let Err(err) = delivery
                        .ack(BasicAckOptions::default())
                        .await
                    {
                        println!("❌ Failed to acknowledge message: {:?}", err);
                    }

                }
                Err(err) => {
                    println!("❌ Error receiving message: {:?}", err);
                }
            }
        }
    });
}