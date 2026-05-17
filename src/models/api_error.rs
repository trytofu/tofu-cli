use reqwest::StatusCode;

use crate::api::PlanLimitApiError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    #[error("not authenticated. run `tofu login`")]
    NotAuthenticated,
    #[error("API returned an unexpected status: {status}")]
    UnexpectedStatus { status: StatusCode },
    #[error("{0}")]
    PlanLimitReached(PlanLimitApiError),
    #[error("{0}")]
    PayloadExpired(String),
}
