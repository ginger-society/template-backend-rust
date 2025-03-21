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

pub async fn create_rabbitmq_pool(rabbitmq_uri: String) -> RabbitMQPool {
    let connection = Connection::connect(&rabbitmq_uri, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");
    Arc::new(Mutex::new(connection))
}