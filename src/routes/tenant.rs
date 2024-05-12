use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::r2d2::ConnectionManager;
use diesel::{insert_into, PgConnection};
use diesel::{r2d2, ExpressionMethods, RunQueryDsl};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::models::identity::schema::Tenant;

/// get tenants documents
#[openapi(tag = "Tenants")]
#[get("/tenant")]
pub fn get_tenants(rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>) -> Json<Vec<Tenant>> {
    // Setting default values

    use crate::models::identity::schema::tenant::dsl::*;

    let mut conn = rdb.get().expect("msg");

    // insert_into(shared_tenant)
    //     .values((name.eq("TName"), is_active.eq(false)))
    //     .execute(&mut conn)
    //     .expect("error inseting");

    let results: Vec<Tenant> = tenant
        .filter(is_active.eq(true))
        .load(&mut conn)
        .expect("Error getting");

    // println!("Displaying {} tenants", results.len());
    // for t in results {
    //     println!("{:?}", t.expiry_date);
    //     println!("-----------\n");
    //     println!("{}", t.id);
    // }

    Json(results)
}
