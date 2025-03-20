#[macro_use]
extern crate rocket;

use db::rabbitmq::DbPool;
use dotenv::dotenv;
use rocket::{Rocket, Build};
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_prometheus::PrometheusMetrics;
use std::env;
use tokio::task;
use crate::db::redis::create_redis_pool;

mod db;
mod fairings;
mod middlewares;
mod models;
mod routes;

const SERVICE_PREFIX: &str = "provisioner";

#[tokio::main]  // ✅ Ensure Tokio runtime is set up before Rocket starts
async fn main() {
    dotenv().ok();
    
    println!("Starting server...");

    let pg_pool = db::connect_rdb();
    let cache_pool = db::redis::create_redis_pool();
    
    
    // ✅ Move async DB connections inside a `tokio::spawn`
    let mongo_handle = task::spawn(async {
        if let (Ok(mongo_uri), Ok(mongo_db_name)) = (env::var("MONGO_URI"), env::var("MONGO_DB_NAME")) {
            println!("Connecting to MongoDB...");
            Some(db::connect_mongo(mongo_uri, mongo_db_name))
        } else {
            println!("Skipping MongoDB connection (missing env variables)");
            None
        }
    });


    let rabbitmq_handle = task::spawn({
        let db_pool = pg_pool.clone(); // ✅ Clone `pg_pool` for RabbitMQ consumer
        let cache_pool = cache_pool.clone();
        async move {
            if let Ok(rabbitmq_uri) = env::var("RABBITMQ_URI") {
                println!("Connecting to RabbitMQ...");
                let rabbitmq_pool = db::rabbitmq::create_rabbitmq_pool(rabbitmq_uri.clone()).await;

                let queue_name = env::var("RABBITMQ_QUEUE_NAME").unwrap_or_else(|_| "default_channel".to_string());

                db::rabbitmq::start_rabbitmq_cluster_message_consumer(rabbitmq_pool.clone(), db_pool, cache_pool, queue_name).await;  // ✅ Pass `db_pool`

                Some(rabbitmq_pool)
            } else {
                println!("Skipping RabbitMQ connection (missing env variable)");
                None
            }
        }
    });

    let prometheus = PrometheusMetrics::new();
    
    let mut server = rocket::build()
        .attach(fairings::cors::CORS)
        .attach(prometheus.clone())
        .mount(
            format!("/{}/", SERVICE_PREFIX),
            openapi_get_routes![routes::index, routes::provisioner::create_cluster, routes::provisioner::reset_lock],
        )
        .mount(
            format!("/{}/api-docs", SERVICE_PREFIX),
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(format!("/{}/metrics", SERVICE_PREFIX), prometheus);


        // ✅ Add PostgreSQL pool to Rocket
    server = server.manage(pg_pool);
    server = server.manage(cache_pool);
    // ✅ Wait for DB connections before attaching them
    if let Ok(Some(mongo_conn)) = mongo_handle.await {
        server = server.manage(mongo_conn);
    }

    if let Ok(Some(rabbit_conn)) = rabbitmq_handle.await {
        server = server.manage(rabbit_conn);
    }

    // ✅ Start Rocket Server
    server.launch().await.expect("Failed to launch Rocket");
}
