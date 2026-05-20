use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Member {
    pub id: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}
