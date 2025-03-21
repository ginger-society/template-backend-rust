use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use lapin::options::BasicPublishOptions;
use lapin::BasicProperties;
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use uuid::Uuid;
use chrono::Utc;
use crate::db::rabbitmq;
use crate::models::schema::{Cluster, ClusterInsertable};
use crate::models::request::CreateClusterRequest;
use crate::models::response::CreateClusterResponse;


/// **Handler: Create a new cluster**
#[openapi()]
#[post("/cluster", data = "<create_request>")]
pub async fn create_cluster(
    db_pool: &State<Pool<ConnectionManager<PgConnection>>>,
    rabbitmq_pool: &State<rabbitmq::RabbitMQPool>,
    create_request: Json<CreateClusterRequest>,
    cache: &State<Pool<RedisConnectionManager>>,
) -> Result<status::Created<Json<CreateClusterResponse>>, status::Custom<String>> {
    use crate::models::schema::schema::cluster::dsl::*;

    let mut conn = db_pool.get().map_err(|_| {
        status::Custom(
            Status::ServiceUnavailable,
            "Failed to get DB connection".to_string(),
        )
    })?;

    let mut cache_conn = cache.get().map_err(|_| {
        status::Custom(Status::ServiceUnavailable, "Failed to get Redis connection".to_string())
    })?;


    let cluster_uuid = Uuid::new_v4().to_string();

    let new_cluster = ClusterInsertable {
        identifier: cluster_uuid.clone(),
        name: create_request.name.clone(),
        parent_server_fqdn: None,
        group_id: "default_group".to_string(),
        description: create_request.description.clone(),
        cluster_ip: None,
        cpu_limit: create_request.cpu_limit,
        ram_limit: create_request.ram_limit,
        state: Some("init".to_string()),
        woskspace_id: create_request.workspace_id.clone(),
        disk_space: create_request.disk_size,
    };

    let created_cluster: Cluster = diesel::insert_into(cluster)
        .values(&new_cluster)
        .get_result::<Cluster>(&mut conn)
        .map_err(|_| {
            status::Custom(
                Status::InternalServerError,
                "Error inserting new cluster".to_string(),
            )
        })?;

    // ✅ RabbitMQ Message
    let message = serde_json::json!({
        "event": "ClusterCreated",
        "identifier": created_cluster.identifier,
        "name": created_cluster.name,
        "cpu_limit": created_cluster.cpu_limit,
        "ram_limit": created_cluster.ram_limit,
        "disk_size": created_cluster.disk_space,
        "timestamp": Utc::now().to_rfc3339(),
    })
    .to_string();

    let queue_name = std::env::var("RABBITMQ_QUEUE_NAME").unwrap_or_else(|_| "default_channel".to_string());

    let rabbitmq_conn = rabbitmq_pool.lock().await;
    let channel = rabbitmq_conn.create_channel().await.map_err(|_| {
        status::Custom(Status::InternalServerError, "Failed to create RabbitMQ channel".to_string())
    })?;

    channel
        .basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            message.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .map_err(|_| {
            status::Custom(Status::InternalServerError, "Failed to publish message".to_string())
        })?;

    

    Ok(status::Created::new("/cluster").body(Json(CreateClusterResponse {
        message: "Cluster created successfully".to_string(),
        id: created_cluster.id,
        identifier: created_cluster.identifier.clone(),
    })))
}


#[openapi()]
#[post("/reset-lock/<id>")]
pub async fn reset_lock(
    id: String,
    cache: &State<Pool<RedisConnectionManager>>,
) -> Result<status::Accepted<String>, status::Custom<String>> {
    let mut cache_conn = cache.get().map_err(|_| {
        status::Custom(Status::ServiceUnavailable, "Failed to get Redis connection".to_string())
    })?;
    
    
    let lock_key = format!("LOCK_{}", id);
    
    let _: () = cache_conn.del(&lock_key).map_err(|_| {
        status::Custom(Status::InternalServerError, "Failed to reset lock".to_string())
    })?;
    
    Ok(status::Accepted("Lock reset successfully".to_string()))
}

#[openapi()]
#[delete("/cluster/<uid>")]
pub async fn delete_cluster_by_identifier(
    uid: String,
    db_pool: &State<Pool<ConnectionManager<PgConnection>>>,
    rabbitmq_pool: &State<rabbitmq::RabbitMQPool>,
) -> Result<status::Accepted<String>, status::Custom<String>> {
    use crate::models::schema::schema::cluster::dsl::*;

    let mut conn = db_pool.get().map_err(|_| {
        status::Custom(Status::ServiceUnavailable, "Failed to get DB connection".to_string())
    })?;

    // ✅ Check if cluster exists
    let existing_cluster: Option<Cluster> = cluster
        .filter(identifier.eq(&uid.clone()))
        .first::<Cluster>(&mut conn)
        .optional()
        .map_err(|_| {
            status::Custom(
                Status::InternalServerError,
                "Error checking cluster existence".to_string(),
            )
        })?;

    if existing_cluster.is_none() {
        return Err(status::Custom(
            Status::NotFound,
            format!("Cluster with identifier '{:?}' not found", uid),
        ));
    }

     // ✅ Update cluster state to "deleting"
     diesel::update(cluster.filter(identifier.eq(&uid)))
     .set(state.eq("deleting"))
     .execute(&mut conn)
     .map_err(|_| {
         status::Custom(
             Status::InternalServerError,
             "Error updating cluster state".to_string(),
         )
     })?;

    // ✅ Send deletion message to RabbitMQ
    let message = serde_json::json!({
        "event": "ClusterDeleted",
        "identifier": uid.clone(),
        "timestamp": Utc::now().to_rfc3339(),
    })
    .to_string();

    let queue_name = std::env::var("DELETE_RABBITMQ_QUEUE_NAME")
        .unwrap_or_else(|_| "delete_channel".to_string());

    let rabbitmq_conn = rabbitmq_pool.lock().await;
    let channel = rabbitmq_conn.create_channel().await.map_err(|_| {
        status::Custom(Status::InternalServerError, "Failed to create RabbitMQ channel".to_string())
    })?;

    channel
        .basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            message.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .map_err(|_| {
            status::Custom(Status::InternalServerError, "Failed to publish delete message".to_string())
        })?;

    println!("✅ Cluster '{:?}' deleted and message sent to queue '{}'", uid, queue_name);

    Ok(status::Accepted(format!(
        "Cluster '{:?}' deleted successfully",
        uid
    )))
}
