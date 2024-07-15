#[macro_use]
extern crate rocket;
use fairings::auth::AuthFairing;
use rocket::Rocket;
use rocket_okapi::settings::UrlObject;

use db::redis::create_redis_pool;
use dotenv::dotenv;
use rocket::Build;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_prometheus::PrometheusMetrics;
use std::env;
use std::process::{exit, Command};
use std::sync::atomic::AtomicUsize;
mod db;
mod errors;
mod fairings;
mod middlewares;
mod models;
mod routes;

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();
    let prometheus = PrometheusMetrics::new();

    let mut server = rocket::build()
        .manage(db::connect_rdb())
        .attach(fairings::cors::CORS)
        .attach(prometheus.clone())
        .attach(AuthFairing)
        .mount("/", openapi_get_routes![routes::index, routes::route2,])
        .mount(
            "/api-docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/metrics", prometheus);

    match env::var("MONGO_URI") {
        Ok(mongo_uri) => match env::var("MONGO_DB_NAME") {
            Ok(mongo_db_name) => {
                println!("Attempting to connect to mongo");
                server = server.manage(db::connect_mongo(mongo_uri, mongo_db_name))
            }
            Err(_) => {
                println!("Not connecting to mongo, missing MONGO_DB_NAME")
            }
        },
        Err(_) => println!("Not connecting to mongo, missing MONGO_URI"),
    };

    match env::var("REDIS_URI") {
        Ok(redis_uri) => {
            println!("Attempting to connect to redis");
            server = server.manage(create_redis_pool(&redis_uri))
        }
        Err(_) => println!("Not connecting to redis"),
    }

    server
}

// Unit testings
#[cfg(test)]
mod tests;
