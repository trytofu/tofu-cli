use reqwest::{Client, Response, StatusCode};
use serde_json::Value;

use crate::{
    api::utils::api_error_from_response,
    models::{
        api_error::ApiError, billing_status::BillingStatus, health_status::HealthStatus, hook::Hook, member::Member, target::Target, user_me::{DeviceLoginPoll, DeviceLoginStart, UserMe}, workspace::Workspace
    },
};

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

    pub async fn me(&self) -> Result<UserMe, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/me", self.base_url);

        let r = self.get_authenticated_response(&url, token).await?;
        let status = r.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(r).await);
        }

        r.json::<UserMe>().await.map_err(ApiError::Request)
    }

    pub async fn start_device_login(&self) -> Result<DeviceLoginStart, ApiError> {
        let url = format!("{}/api/device-login/start", self.base_url);
        let body = serde_json::json!({"client_name": "Tofu CLI"});

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::UnexpectedStatus { status });
        }

        response
            .json::<DeviceLoginStart>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn poll_device_login(&self, device_code: &str) -> Result<DeviceLoginPoll, ApiError> {
        let url = format!("{}/api/device-login/poll", self.base_url);
        let body = serde_json::json!({ "device_code": device_code });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::UnexpectedStatus { status });
        }

        response
            .json::<DeviceLoginPoll>()
            .await
            .map_err(ApiError::Request)
    }

    async fn get_authenticated_response(
        &self,
        url: &str,
        token: &str,
    ) -> Result<Response, ApiError> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(response).await);
        }

        Ok(response)
    }

    async fn post_authenticated_response(
        &self,
        url: &str,
        token: &str,
        body: &Value,
    ) -> Result<Response, ApiError> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(response).await);
        }

        Ok(response)
    }

    async fn put_authenticated_response(
        &self,
        url: &str,
        token: &str,
        body: &Value,
    ) -> Result<Response, ApiError> {
        let response = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(response).await);
        }
        Ok(response)
    }
}

/// Billing status
impl ApiClient {
    pub async fn billing_status(&self) -> Result<BillingStatus, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/billing/status", self.base_url);
        let r = self.get_authenticated_response(&url, token).await?;

        r.json::<BillingStatus>().await.map_err(ApiError::Request)
    }
}

/// Workspace
impl ApiClient {
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces", self.base_url);
        let response = self.get_authenticated_response(&url, token).await?;

        response
            .json::<Vec<Workspace>>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn set_active_workspace(&self, workspace_id: &str) -> Result<UserMe, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/me/active-workspace", self.base_url);
        let body = serde_json::json!({"workspace_id": workspace_id});

        let response = self.put_authenticated_response(&url, token, &body).await?;

        response.json::<UserMe>().await.map_err(ApiError::Request)
    }

    pub async fn create_workspace(
        &self,
        name: String,
        slug: String,
    ) -> Result<Workspace, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces", self.base_url);
        let body = serde_json::json!({ "name": name, "slug": slug });
        let response = self.post_authenticated_response(&url, token, &body).await?;

        response
            .json::<Workspace>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn list_members(&self, workspace_id: &str) -> Result<Vec<Member>, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces/{workspace_id}/members", self.base_url);

        let response = self.get_authenticated_response(&url, token).await?;
        response
            .json::<Vec<Member>>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn add_member(&self, workspace_id: &str, email: String) -> Result<(), ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces/{workspace_id}/members", self.base_url);

        let body = serde_json::json!({ "email": email, "role": "member" });
        self.post_authenticated_response(&url, token, &body).await?;

        Ok(())
    }
}

// Hooks
impl ApiClient {
    pub async fn list_hooks(&self, workspace_id: &str) -> Result<Vec<Hook>, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces/{workspace_id}/hooks", self.base_url);

        let response = self.get_authenticated_response(&url, token).await?;
        response
            .json::<Vec<Hook>>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn create_hook(
        &self,
        workspace_id: &str,
        name: String,
        slug: String,
    ) -> Result<Hook, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces/{workspace_id}/hooks", self.base_url);
        let body = serde_json::json!({ "name": name, "slug": slug });

        let response = self.post_authenticated_response(&url, token, &body).await?;
        response.json::<Hook>().await.map_err(ApiError::Request)
    }

    #[allow(dead_code)]
    pub async fn get_hook(&self, hook_id: &str) -> Result<Hook, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/hooks/{hook_id}", self.base_url);

        let response = self.get_authenticated_response(&url, token).await?;
        response.json::<Hook>().await.map_err(ApiError::Request)
    }
}

// Targets
impl ApiClient {
    pub async fn list_targets(&self, hook_id: &str) -> Result<Vec<Target>, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/hooks/{hook_id}/targets", self.base_url);
        let r = self.get_authenticated_response(&url, token).await?;
        r.json::<Vec<Target>>().await.map_err(ApiError::Request)
    }
}