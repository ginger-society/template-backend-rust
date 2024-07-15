use chrono::{NaiveDate, Utc};
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::r2d2::ConnectionManager;
use diesel::{insert_into, PgConnection};
use diesel::{r2d2, ExpressionMethods, RunQueryDsl};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::models::schema::{Student, StudentInsertable};

/// get all students
#[openapi(tag = "Students")]
#[get("/student")]
pub fn get_students(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> Json<Vec<Student>> {
    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("Failed to get DB connection");

    let results: Vec<Student> = student.load(&mut conn).expect("Error fetching students");

    println!("Displaying {} students", results.len());

    Json(results)
}

/// get student by id
#[openapi(tag = "Students")]
#[get("/student/<student_id>")]
pub fn get_student_by_id(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    student_id: i64,
) -> Json<Student> {
    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("Failed to get DB connection");

    let result: Student = student
        .filter(id.eq(student_id))
        .first(&mut conn)
        .expect("Error fetching student");

    Json(result)
}

/// create a new student
#[openapi(tag = "Students")]
#[post("/student", data = "<new_student>")]
pub fn post_student(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    new_student: Json<StudentInsertable>,
) -> Json<String> {
    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("Failed to get DB connection");

    insert_into(student)
        .values((
            name.eq(&new_student.name),
            roll_number.eq(&new_student.roll_number),
            on_scholarship.eq(new_student.on_scholarship),
            father_name.eq(&new_student.father_name),
            address.eq(&new_student.address),
            data_of_birth.eq(new_student.data_of_birth),
            created_at.eq(Utc::now().naive_utc()),
            updated_at.eq(Utc::now().naive_utc().date()),
            has_cab_service.eq(new_student.has_cab_service),
        ))
        .execute(&mut conn)
        .expect("Error inserting student");

    Json("Student created successfully".to_string())
}

/// update student by id
#[openapi(tag = "Students")]
#[patch("/student/<student_id>", data = "<updated_student>")]
pub fn patch_student_by_id(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    student_id: i64,
    updated_student: Json<Student>,
) -> Json<String> {
    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("Failed to get DB connection");

    diesel::update(student.filter(id.eq(student_id)))
        .set((
            name.eq(&updated_student.name),
            roll_number.eq(&updated_student.roll_number),
            on_scholarship.eq(updated_student.on_scholarship),
            father_name.eq(&updated_student.father_name),
            address.eq(&updated_student.address),
            data_of_birth.eq(updated_student.data_of_birth),
            updated_at.eq(Utc::now().naive_utc().date()),
            has_cab_service.eq(updated_student.has_cab_service),
        ))
        .execute(&mut conn)
        .expect("Error updating student");

    Json("Student updated successfully".to_string())
}

/// delete student by id
#[openapi(tag = "Students")]
#[delete("/student/<student_id>")]
pub fn delete_student_by_id(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    student_id: i64,
) -> Json<String> {
    use crate::models::schema::schema::student::dsl::*;

    let mut conn = rdb.get().expect("Failed to get DB connection");

    diesel::delete(student.filter(id.eq(student_id)))
        .execute(&mut conn)
        .expect("Error deleting student");

    Json("Student deleted successfully".to_string())
}
