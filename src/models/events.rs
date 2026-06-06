use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EventListItem {
    pub id: String,
    pub hook_id: String,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub body_preview: Option<String>,
    pub received_at: String,
    pub payload_expires_at: String,
    pub metadata_expires_at: String,
    pub payload_expired_at: Option<String>,
    pub manually_expired_at: Option<String>,
    pub payload_expired: bool,
    pub replay_available: bool,
    pub delivery_summary: DeliverySummary,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverySummary {
    pub total: i64,
    pub success: i64,
    pub failed: i64,
    pub pending: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventDetail {
    pub id: String,
    pub hook_id: String,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub headers: serde_json::Value,
    pub body_preview: Option<String>,
    pub received_at: String,
    pub payload_expires_at: String,
    pub metadata_expires_at: String,
    pub payload_expired_at: Option<String>,
    pub manually_expired_at: Option<String>,
    pub payload_expired: bool,
    pub replay_available: bool,
    pub deliveries: Vec<DeliveryDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliveryDetail {
    pub id: String,
    pub target_name: String,
    pub target_url: String,
    pub status: String,
    pub response_status: Option<i32>,
    pub response_body_preview: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub replayed: bool,
    pub attempted_at: String,
}
