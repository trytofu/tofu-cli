use tokio::time::{Duration, sleep};

use crate::{
    api::ApiClient,
    config::Config,
    models::{
        api_error::ApiError,
        user_me::{DeviceLoginStart, UserMe},
    },
    utils::{os, output, time},
};

pub async fn login(
    config: &mut Config,
    token: Option<String>,
    api_base_url: Option<String>,
    no_browser: bool,
    json: bool,
) {
    match token {
        Some(t) => login_with_token(config, t, api_base_url, json).await,
        None => device_login(config, api_base_url, no_browser, json).await,
    }
}

pub fn logout(config: &mut Config, json: bool) {
    config.token = None;

    if let Err(e) = config.save() {
        output::error(format!("Failed to save config: {e}"));
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "ok"
            })
        )
    } else {
        output::success("Logged out.");
    }
}

pub async fn whoami(client: &ApiClient, json: bool) {
    match fetch_current_user(client).await {
        Ok(user) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": user.id,
                        "email": user.email,
                        "created_at": user.created_at,
                    })
                );
            } else {
                println!(
                    "{}",
                    output::kv_table(vec![
                        ("Email", user.email),
                        ("User ID", user.id),
                        ("Created", time::fmt_time(&user.created_at)),
                    ])
                );
            }
        }
        Err(ApiError::NotAuthenticated) => {
            output::error("Not authenticated.");
            output::command("Try running `tofu login`");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            output::error("Invalid token.");
            output::warning("Run `tofu login` to re-authenticate.");
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to fetch user: {e}"));
            std::process::exit(1);
        }
    }
}

async fn login_with_token(
    config: &mut Config,
    token: String,
    api_base_url: Option<String>,
    json: bool,
) {
    complete_login(config, token, api_base_url, json).await
}

async fn device_login(
    config: &mut Config,
    api_base_url: Option<String>,
    no_browser: bool,
    json: bool,
) {
    let base_url = config.resolve_api_base_url(api_base_url.clone());
    let client = ApiClient::new(base_url.clone(), None);

    let started = match client.start_device_login().await {
        Ok(s) => s,
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::NOT_FOUND => {
            output::error("This Tofu API does not support device login yet.");
            output::command("Try using `tofu login --token <token>`");
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to start device login: {e}"));
            std::process::exit(1);
        }
    };

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "pending",
                "verification_uri": started.verification_uri,
                "verification_uri_complete": started.verification_uri_complete,
                "user_code": started.user_code,
                "expires_in": started.expires_in,
            })
        );
    } else {
        output::info("Open this URL to approve Tofu CLI login:");
        output::info(&started.verification_uri_complete);
        println!();
        println!("Code: {}", started.user_code);
    }

    if !no_browser {
        let _ = os::open_browser(&started.verification_uri_complete);
    }

    let token = poll_until_approved(&client, &started).await;
    complete_login(config, token, api_base_url, json).await
}

async fn poll_until_approved(client: &ApiClient, started: &DeviceLoginStart) -> String {
    let interval = started.interval.clamp(1, 10) as u64;
    let max_attempts = ((started.expires_in.max(1) as u64) / interval).saturating_add(1);

    for _ in 0..max_attempts {
        sleep(Duration::from_secs(interval)).await;

        match client.poll_device_login(&started.device_code).await {
            Ok(poll) if poll.status == "approved" => match poll.token {
                Some(token) => return token,
                None => {
                    output::error("Device login was approved, but no token was returned.");
                    std::process::exit(1);
                }
            },
            Ok(poll) if poll.status == "pending" => {}
            Ok(poll) if poll.status == "denied" => {
                output::error("Device login denied.");
                std::process::exit(1);
            }
            Ok(poll) if poll.status == "expired" => {
                output::error("Device login expired.");
                output::command("Try running `tofu login` again.");
                std::process::exit(1);
            }
            Ok(poll) => {
                output::error(format!(
                    "Device login returned unexpected status: {}",
                    poll.status
                ));
                std::process::exit(1);
            }
            Err(e) => {
                output::error(format!("Failed while polling device login: {e}"));
                std::process::exit(1);
            }
        }
    }

    output::error("Device login timed out.");
    output::command("Try running `tofu login` again.");
    std::process::exit(1);
}

async fn complete_login(
    config: &mut Config,
    token: String,
    api_base_url: Option<String>,
    json: bool,
) {
    let base_url = config.resolve_api_base_url(api_base_url.clone());
    let client = ApiClient::new(base_url, Some(token.clone()));
    let user = match fetch_current_user(&client).await {
        Ok(user) => user,
        Err(ApiError::NotAuthenticated) => {
            output::error("Not authenticated.");
            output::command("Try running `tofu login`");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            output::error("Invalid token.");
            output::warning("Check your token and try again.");
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Login failed: {e}"));
            std::process::exit(1);
        }
    };

    config.token = Some(token);
    if let Some(url) = api_base_url {
        config.api_base_url = Some(url);
    }

    if let Err(e) = config.save() {
        output::error(format!("Failed to save config: {e}"));
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "ok",
                "email": user.email,
            })
        );
    } else {
        output::success(format!("Logged in as {}", user.email));
    }
}

async fn fetch_current_user(client: &ApiClient) -> Result<UserMe, ApiError> {
    client.me().await
}
