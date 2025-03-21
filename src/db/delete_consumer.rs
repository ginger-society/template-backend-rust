use futures::TryStreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, Consumer,
};
use r2d2_redis::RedisConnectionManager;
use serde_json::Value;
use std::sync::Arc;
use tokio::{sync::Mutex, time::{sleep, Duration}};
use diesel::{r2d2::ConnectionManager, PgConnection, r2d2::Pool};
use crate::db::cluster_helper::{delete_cluster, release_compute_unit_lock, update_cluster_state};

pub type RabbitMQPool = Arc<Mutex<Connection>>;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub async fn start_rabbitmq_cluster_deletion_consumer(
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

    // ✅ Declare queue
    if let Err(err) = channel
        .queue_declare(&queue_name, QueueDeclareOptions::default(), FieldTable::default())
        .await
    {
        eprintln!("❌ Failed to declare queue '{}': {:?}", queue_name, err);
        return;
    }

    match channel.queue_declare(
        &format!("{}_waiting_queue", queue_name.clone()),
        QueueDeclareOptions::default(),
        {
            let mut args = FieldTable::default();
            args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("".into())); // Default exchange
            args.insert("x-dead-letter-routing-key".into(), AMQPValue::LongString(queue_name.clone().into())); // Move to main queue
            args.insert("x-message-ttl".into(), AMQPValue::LongInt(1200000)); // 1200s delay
            args
        },
    ).await{
        Ok(_) => {println!("Waiting queue declared")},
        Err(_) => {println!("error creating waiting queue")},
    };


    let consumer: Consumer = match channel
        .basic_consume(&queue_name.clone(), "delete_consumer", BasicConsumeOptions::default(), FieldTable::default())
        .await
    {
        Ok(consumer) => consumer,
        Err(err) => {
            eprintln!("❌ Failed to start consumer: {:?}", err);
            return;
        }
    };

    println!("✅ Deletion consumer started on queue '{}'", queue_name);

    let channel_clone = channel.clone();

    tokio::spawn(async move {
        let mut consumer_stream = consumer.into_stream();

        while let Some(delivery_result) = futures::TryStreamExt::try_next(&mut consumer_stream).await.unwrap_or(None) {
            let channel_clone = channel_clone.clone();
            let db_pool = db_pool.clone();
            let cache_pool = cache_pool.clone();

            tokio::spawn(async move {
                if let Err(err) = handle_delete_cluster_message(delivery_result, &channel_clone, &db_pool, &cache_pool).await {
                    eprintln!("❌ Error processing deletion message: {:?}", err);
                }
            });
        }
    });
}

async fn handle_delete_cluster_message(
    delivery: lapin::message::Delivery,
    channel: &Channel,
    db_pool: &DbPool,
    cache_pool: &Pool<RedisConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = String::from_utf8_lossy(&delivery.data);
    println!("📩 Received delete message: {}", message);

    let parsed_message: Result<Value, _> = serde_json::from_str(&message);
    if let Ok(json_data) = parsed_message {
        let cluster_name = json_data["name"].as_str().unwrap_or("unknown-cluster");
        let mut cache_conn = cache_pool.get().expect("Failed to get cache connection");

        match delete_cluster(&db_pool, cluster_name).await {
            Ok(Some(unit)) => {
                println!("✅ Cluster '{}' deleted successfully.", cluster_name);
                release_compute_unit_lock(&mut cache_conn, unit.id);
                update_cluster_state(db_pool, cluster_name, "deleted").await?;
            }
            Ok(None) => {
                println!("⚠️ Cluster '{}' not found.", cluster_name);
            }
            Err(err) => {
                eprintln!("❌ Failed to delete cluster '{}': {:?}", cluster_name, err);
            }
        }
    } else {
        eprintln!("❌ Invalid delete message format: {}", message);
    }

    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}