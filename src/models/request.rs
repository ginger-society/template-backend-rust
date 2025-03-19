use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateClusterRequest {
    pub name: String,
    pub description: String,
    pub cpu_limit: f64,
    pub ram_limit: f64,
    pub disk_size: i32,
    pub workspace_id: String,
    pub region_code: String
}