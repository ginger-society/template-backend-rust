use crate::models::identity::schema::{administrator, Administrator, Test};
use crate::models::response::MessageResponse;
use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{r2d2, RunQueryDsl};
use diesel::{ExpressionMethods, PgConnection};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

pub mod customer;
pub mod tenant;

/// This is a description. <br />You can do simple html <br /> like <b>this<b/>
#[openapi(tag = "Hello World")]
#[get("/")]
pub fn index(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: &State<Pool<RedisConnectionManager>>,
) -> Json<MessageResponse> {
    use crate::models::identity::schema::administrator::dsl::*;
    use crate::models::identity::schema::test::dsl::*;

    let mut conn = match rdb.get() {
        Ok(conn) => conn,
        Err(_) => {
            return Json(MessageResponse {
                message: "Hello World!".to_string(),
            })
        }
    };

    println!("{:?}", cache);

    let mut cache_connection = match cache.get() {
        Ok(conn) => conn,
        Err(_) => {
            return Json(MessageResponse {
                message: "Error getting redis cnnection".to_string(),
            })
        }
    };

    let _: () = cache_connection
        .set("my_key", String::from("value"))
        .unwrap();

    // add example to use cache_connection
    let v: String = cache_connection.get("my_key").unwrap();

    println!("Value: {:?}", v);

    let results: Vec<Test> = test
        .filter(field3.eq(true))
        .load(&mut conn)
        .expect("Error loading posts");

    let t: Test = test.first(&mut conn).expect("Error loading post");
    println!("{:?}", t);
    let t2: Administrator = administrator.first(&mut conn).expect("Error loading post");
    println!("t2 {:?}", t2);

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.field3);
        println!("-----------\n");
        println!("{}", post.id);
    }

    Json(MessageResponse {
        message: "Hello World!".to_string(),
    })
}
