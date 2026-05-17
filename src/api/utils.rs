use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::models::api_error::ApiError;

pub async fn api_error_from_response(response: reqwest::Response) -> ApiError {
    let status = response.status();

    if status == StatusCode::PAYMENT_REQUIRED || status == StatusCode::GONE {
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(err) => return ApiError::Request(err),
        };

        if status == StatusCode::PAYMENT_REQUIRED
            && let Ok(env) = serde_json::from_slice::<PlanLimitEnv>(&bytes)
            && env.error.code == "plan_limit_reached"
        {
            return ApiError::PlanLimitReached(env.error);
        }

        if let Ok(env) = serde_json::from_slice::<ErrorEnv>(&bytes)
            && env.error.code == "payload_expired"
        {
            return ApiError::PayloadExpired(env.error.message);
        }
    }

    ApiError::UnexpectedStatus { status }
}

#[derive(Debug, Deserialize)]
struct PlanLimitEnv {
    error: PlanLimitApiError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlanLimitApiError {
    pub code: String,
    pub message: String,
    pub upgrade_url: String,
    pub limit: PlanLimitDetail,
}

impl std::fmt::Display for PlanLimitApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} (upgrade at {})",
            self.code, self.message, self.upgrade_url
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlanLimitDetail {
    pub resource: String,
    pub allowed: i64,
    pub current: i64,
    pub plan: String,
}

#[derive(Debug, Deserialize)]
struct ErrorEnv {
    error: ApiErrorBody,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    code: String,
    message: String,
}
