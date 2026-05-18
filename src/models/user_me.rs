use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserMe {
    pub id: String,
    pub email: String,
    pub created_at: String,
    #[serde(default)] #[allow(dead_code)]
    pub active_workspace_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceLoginStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Debug, Deserialize)]
pub struct DeviceLoginPoll {
    pub status: String,
    pub token: Option<String>,
}
