use reqwest::Client;

use crate::models::{api_error::ApiError, health_status::HealthStatus};

pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    pub async fn health(&self) -> Result<HealthStatus, ApiError> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(ApiError::Request)?;

        if response.status().is_success() {
            Ok(HealthStatus::Ok)
        } else {
            Ok(HealthStatus::NotOk(response.status()))
        }
    }
}
