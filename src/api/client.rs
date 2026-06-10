use reqwest::{Client, Response, StatusCode};
use serde_json::Value;

use crate::{
    api::utils::api_error_from_response,
    models::{
        api_error::ApiError,
        billing_status::BillingStatus,
        events::{EventDetail, EventListItem},
        health_status::HealthStatus::{self},
        hook::Hook,
        member::Member,
        target::Target,
        user_me::{DeviceLoginPoll, DeviceLoginStart, UserMe},
        workspace::Workspace,
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
        body: Option<&Value>,
    ) -> Result<Response, ApiError> {
        let request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {token}"));
        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        };
        let response = request.send().await.map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(response).await);
        }

        Ok(response)
    }

    async fn patch_authenticated_response(
        &self,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Response, ApiError> {
        let request = self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {token}"));
        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        };
        let response = request.send().await.map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(api_error_from_response(response).await);
        }

        Ok(response)
    }

    async fn delete_authenticated_response(&self, url: &str, token: &str) -> Result<(), ApiError> {
        let response = self
            .client
            .delete(url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(ApiError::Request)?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::UnexpectedStatus { status });
        }

        if !status.is_success() {
            return Err(ApiError::UnexpectedStatus { status });
        }

        Ok(())
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
    pub async fn get_workspace(&self, workspace_id: &str) -> Result<Workspace, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/workspaces/{workspace_id}", self.base_url);
        let r = self.get_authenticated_response(&url, token).await?;
        r.json().await.map_err(ApiError::Request)
    }

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
        let response = self
            .post_authenticated_response(&url, token, Some(&body))
            .await?;

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
        self.post_authenticated_response(&url, token, Some(&body))
            .await?;

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

        let response = self
            .post_authenticated_response(&url, token, Some(&body))
            .await?;
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

    pub async fn disable_target(&self, target_id: &str) -> Result<Target, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/targets/{target_id}/disable", self.base_url);

        let r = self.post_authenticated_response(&url, token, None).await?;
        r.json::<Target>().await.map_err(ApiError::Request)
    }

    pub async fn enable_target(&self, target_id: &str) -> Result<Target, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/targets/{target_id}/enable", self.base_url);

        let r = self.post_authenticated_response(&url, token, None).await?;
        r.json::<Target>().await.map_err(ApiError::Request)
    }

    pub async fn delete_target(&self, target_id: &str) -> Result<(), ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/targets/{target_id}", self.base_url);
        self.delete_authenticated_response(&url, token).await
    }

    pub async fn create_target(
        &self,
        hook_id: &str,
        name: String,
        url: String,
        enabled: bool,
    ) -> Result<Target, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let body = serde_json::json!({ "name": name, "url": url, "enabled": enabled });

        let r = self
            .post_authenticated_response(
                &format!("{}/api/hooks/{hook_id}/targets", self.base_url),
                token,
                Some(&body),
            )
            .await?;
        r.json::<Target>().await.map_err(ApiError::Request)
    }

    pub async fn update_target(
        &self,
        target_id: &str,
        name: Option<String>,
        url: Option<String>,
        enabled: Option<bool>,
    ) -> Result<Target, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url_path = format!("{}/api/targets/{target_id}", self.base_url);

        let mut body = serde_json::Map::new();
        if let Some(n) = name {
            body.insert("name".to_string(), serde_json::Value::String(n));
        }
        if let Some(u) = url {
            body.insert("url".to_string(), serde_json::Value::String(u));
        }
        if let Some(e) = enabled {
            body.insert("enabled".to_string(), serde_json::Value::Bool(e));
        }

        let r = self
            .patch_authenticated_response(&url_path, token, Some(&body.into()))
            .await?;
        r.json::<Target>().await.map_err(ApiError::Request)
    }
}

// Events
impl ApiClient {
    pub async fn list_events(
        &self,
        hook_id: &str,
        limit: u32,
    ) -> Result<Vec<EventListItem>, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/hooks/{hook_id}/events?limit={limit}", self.base_url);

        let r = self.get_authenticated_response(&url, token).await?;
        r.json::<Vec<EventListItem>>()
            .await
            .map_err(ApiError::Request)
    }

    pub async fn get_event(&self, event_id: &str) -> Result<EventDetail, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/events/{event_id}", self.base_url);

        let r = self.get_authenticated_response(&url, token).await?;
        r.json().await.map_err(ApiError::Request)
    }

    pub async fn expire_event(&self, event_id: &str) -> Result<EventDetail, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/events/{event_id}/expire", self.base_url);

        let r = self.post_authenticated_response(&url, token, None).await?;
        r.json().await.map_err(ApiError::Request)
    }
}

// Replay
impl ApiClient {
    pub async fn replay_event_to_target(
        &self,
        event_id: &str,
        target_id: &str,
    ) -> Result<(), ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/events/{event_id}/replay/{target_id}", self.base_url);
        self.post_authenticated_response(&url, token, None).await?;
        Ok(())
    }

    pub async fn replay_event(&self, event_id: &str) -> Result<(), ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/events/{event_id}/replay", self.base_url);
        self.post_authenticated_response(&url, token, None).await?;
        Ok(())
    }
}

impl ApiClient {
    pub async fn stream_events(&self, hook_id: &str) -> Result<reqwest::Response, ApiError> {
        let token = self.token.as_ref().ok_or(ApiError::NotAuthenticated)?;
        let url = format!("{}/api/hooks/{hook_id}/events/stream", self.base_url);

        let r = self.get_authenticated_response(&url, token).await?;
        Ok(r)
    }
}
