#[macro_use]
extern crate rocket;

use db::redis::create_redis_pool;
use dotenv::dotenv;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_prometheus::PrometheusMetrics;
use std::process::{exit, Command};

mod db;
mod errors;
mod fairings;
mod models;
mod request_guards;
mod routes;

#[launch]
fn rocket() -> _ {
    // Call the `db-compose render --skip` command
    let output = Command::new("db-compose")
        .arg("render")
        .arg("--skip")
        .output()
        .expect("Failed to execute `db-compose render --skip`");

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    } else {
        println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    }

    dotenv().ok();
    let prometheus = PrometheusMetrics::new();

    rocket::build()
        .attach(db::init())
        .manage(db::connect_rdb())
        .manage(create_redis_pool("redis://127.0.0.1/"))
        .attach(fairings::cors::CORS)
        .attach(prometheus.clone())
        .mount(
            "/",
            openapi_get_routes![
                routes::index,
                routes::customer::get_customers,
                routes::customer::get_customer_by_id,
                routes::customer::post_customer,
                routes::customer::patch_customer_by_id,
                routes::customer::delete_customer_by_id,
                routes::student::get_students
            ],
        )
        .mount(
            "/api-docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/metrics", prometheus)
}

// Unit testings
#[cfg(test)]
mod tests;
