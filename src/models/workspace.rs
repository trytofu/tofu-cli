use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub hook_count: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}
