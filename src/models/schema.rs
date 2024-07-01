#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use chrono::offset::Utc;
use chrono::DateTime;
use chrono::NaiveDate;
use diesel::Associations;
use diesel::Identifiable;
use diesel::{deserialize::Queryable, table, Selectable};
use schemars::JsonSchema;
use serde::Serialize;

table! {
    student (id) {
        #[max_length = 150]
        name ->Varchar,
        #[max_length = 40]
        roll_number ->Varchar,
        on_scholarship ->Bool,
        #[max_length = 100]
        father_name ->Nullable<Varchar>,
        #[max_length = 500]
        address ->Varchar,
        data_of_birth ->Nullable<Date>,
        created_at ->Timestamptz,
        updated_at ->Date,
        has_cab_service ->Nullable<Bool>,
        id ->BigInt,

    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema, Identifiable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = student)]
pub struct student {
    pub name: String,
    pub roll_number: String,
    pub on_scholarship: bool,
    pub father_name: Option<String>,
    pub address: String,
    pub data_of_birth: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: NaiveDate,
    pub has_cab_service: Option<bool>,
    pub id: i64,
}

table! {
    enrollment (id) {
        student_id ->BigInt,
        course_id ->Nullable<BigInt>,
        id ->BigInt,

    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema, Identifiable, Associations)]
#[diesel(belongs_to(student, foreign_key = student_id))]
#[diesel(belongs_to(course, foreign_key = course_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = enrollment)]
pub struct enrollment {
    pub student_id: i64,
    pub course_id: Option<i64>,
    pub id: i64,
}

table! {
    course (id) {
        #[max_length = 100]
        name ->Varchar,
        course_type ->Varchar,
        duration ->Nullable<Integer>,
        id ->BigInt,

    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema, Identifiable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = course)]
pub struct course {
    pub name: String,
    pub course_type: String,
    pub duration: Option<i32>,
    pub id: i64,
}

diesel::joinable!(enrollment -> student (student_id));
diesel::joinable!(enrollment -> course (course_id));

diesel::allow_tables_to_appear_in_same_query!(student, enrollment, course,);
