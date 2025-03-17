use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateClusterRequest {
    pub name: String,
    pub parent_server_fqdn: String,
    pub identify_file_name: String,
    pub group_id: String,
    pub description: String,
    pub cluster_ip: String,
    pub cpu_limit: f64,
    pub ram_limit: f64,
    pub state: Option<String>,
    pub workspace_id: String,
    pub ipv4:String
}