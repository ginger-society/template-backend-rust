use chrono::{NaiveDate, Utc};
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::r2d2::ConnectionManager;
use diesel::{insert_into, PgConnection};
use diesel::{r2d2, ExpressionMethods, RunQueryDsl};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::models::schema::Student;

/// get tenants documents
#[openapi(tag = "Students")]
#[get("/student")]
pub fn get_students(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> Json<Vec<Student>> {
    // Setting default values

    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("msg");

    insert_into(student)
        .values((
            name.eq("TName"),
            has_cab_service.eq(true),
            roll_number.eq("12345"),
            on_scholarship.eq(false),
            father_name.eq(Some("Father Name")),
            address.eq("123 Address St."),
            created_at.eq(Utc::now().naive_utc()),
            updated_at.eq(NaiveDate::from_ymd_opt(2024, 7, 2).unwrap()),
            data_of_birth.eq(Some(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())),
        ))
        .execute(&mut conn)
        .expect("error inseting");

    let results: Vec<Student> = student
        .filter(has_cab_service.eq(true))
        .load(&mut conn)
        .expect("Error getting");

    println!("Displaying {} tenants", results.len());
    // for t in results {
    //     println!("{:?}", t.data_of_birth);
    //     println!("-----------\n");
    //     println!("{}", t.id);
    // }

    Json(results)
}
