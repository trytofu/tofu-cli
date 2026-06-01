use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Hook {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub slug: String,
    pub provider_url: String,
    pub created_at: String,
    pub updated_at: String,
}
