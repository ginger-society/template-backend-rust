#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use rocket::{Rocket, Build};
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_prometheus::PrometheusMetrics;
use std::env;
use tokio::task;

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

    let redis_handle = task::spawn(async {
        if let Ok(redis_uri) = env::var("REDIS_URI") {
            println!("Connecting to Redis...");
            Some(db::redis::create_redis_pool(redis_uri))
        } else {
            println!("Skipping Redis connection (missing env variable)");
            None
        }
    });

    let rabbitmq_handle = task::spawn(async {
        if let Ok(rabbitmq_uri) = env::var("RABBITMQ_URI") {
            println!("Connecting to RabbitMQ...");
            Some(db::rabbitmq::create_rabbitmq_pool(rabbitmq_uri).await)
        } else {
            println!("Skipping RabbitMQ connection (missing env variable)");
            None
        }
    });

    let prometheus = PrometheusMetrics::new();
    
    let mut server = rocket::build()
        .attach(fairings::cors::CORS)
        .attach(prometheus.clone())
        .mount(
            format!("/{}/", SERVICE_PREFIX),
            openapi_get_routes![routes::index, routes::provisioner::create_cluster],
        )
        .mount(
            format!("/{}/api-docs", SERVICE_PREFIX),
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(format!("/{}/metrics", SERVICE_PREFIX), prometheus);

        // ✅ Create a PostgreSQL connection pool
    let pg_pool = db::connect_rdb();

        // ✅ Add PostgreSQL pool to Rocket
    server = server.manage(pg_pool);

    // ✅ Wait for DB connections before attaching them
    if let Ok(Some(mongo_conn)) = mongo_handle.await {
        server = server.manage(mongo_conn);
    }
    if let Ok(Some(redis_conn)) = redis_handle.await {
        server = server.manage(redis_conn);
    }
    if let Ok(Some(rabbit_conn)) = rabbitmq_handle.await {
        server = server.manage(rabbit_conn);
    }

    // ✅ Start Rocket Server
    server.launch().await.expect("Failed to launch Rocket");
}
