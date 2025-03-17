use futures::TryStreamExt;
use lapin::{
    options::BasicConsumeOptions, types::FieldTable, Connection, ConnectionProperties, Consumer,
};
use std::sync::Arc;
use tokio::{sync::Mutex, task};
use tokio_stream::StreamExt; // ✅ Required for `.next()`

pub type RabbitMQPool = Arc<Mutex<Connection>>;

pub async fn create_rabbitmq_pool(rabbitmq_uri: String) -> RabbitMQPool {
    let connection = Connection::connect(&rabbitmq_uri, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");
    Arc::new(Mutex::new(connection))
}

// Function to start consuming messages
pub async fn start_rabbitmq_consumer(rabbitmq_pool: RabbitMQPool, queue_name: String) {
    let connection = rabbitmq_pool.lock().await;
    let channel = connection.create_channel().await.expect("Failed to create channel");

    let consumer: Consumer = channel
        .basic_consume(
            &queue_name, // ✅ Use a reference here
            "consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to start consumer");

    task::spawn(async move {
        println!("Waiting for messages on queue: {}", queue_name); // ✅ queue_name now owned
        let mut consumer_stream = consumer.into_stream(); // Convert Consumer to Stream

        while let Some(delivery_result) = consumer_stream.next().await {
            match delivery_result {
                Ok(delivery) => {
                    let message = String::from_utf8_lossy(&delivery.data);
                    println!("Received message: {}", message);

                    // Acknowledge the message
                    if let Err(err) = delivery
                        .ack(lapin::options::BasicAckOptions::default())
                        .await
                    {
                        println!("Failed to acknowledge message: {:?}", err);
                    }
                }
                Err(err) => {
                    println!("Error receiving message: {:?}", err);
                }
            }
        }
    });
}
