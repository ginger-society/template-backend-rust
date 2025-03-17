use chrono::{DateTime, NaiveDate, Utc};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateClusterResponse {
    pub message: String,
    pub id: i64,
    pub identifier: String,
}