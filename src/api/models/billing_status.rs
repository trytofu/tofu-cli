use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BillingStatus {
    pub plan: String,
    pub status: String,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub usage: BillingUsage,
    pub limits: BillingLimits,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BillingUsage {
    pub hooks: i64,
    pub targets_per_hook: i64,
    pub events_this_month: i64,
    pub workspace_members: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BillingLimits {
    pub hooks: i64,
    pub targets_per_hook: i64,
    pub events_this_month: i64,
    pub retention_days: i64,
    pub workspace_members: i64,
}
