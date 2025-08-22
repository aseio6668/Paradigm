/// Core API functionality and utilities
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API versioning
pub const CURRENT_API_VERSION: &str = "v1";
pub const SUPPORTED_VERSIONS: &[&str] = &["v1"];

/// API metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub documentation_url: String,
    pub support_email: String,
    pub terms_of_service_url: String,
    pub privacy_policy_url: String,
}

impl Default for ApiInfo {
    fn default() -> Self {
        Self {
            name: "Paradigm API".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Enterprise REST API for Paradigm blockchain network".to_string(),
            documentation_url: "https://docs.paradigm.network/api".to_string(),
            support_email: "support@paradigm.network".to_string(),
            terms_of_service_url: "https://paradigm.network/terms".to_string(),
            privacy_policy_url: "https://paradigm.network/privacy".to_string(),
        }
    }
}

/// Request context passed through middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            user_id: None,
            api_key_id: None,
            client_ip: "unknown".to_string(),
            user_agent: None,
            started_at: chrono::Utc::now(),
        }
    }
}