use crate::{
    api::ApiClient,
    models::api_error::ApiError,
    utils::output::{self, print_usage},
};

pub async fn run(client: &ApiClient, json: bool) {
    match client.billing_status().await {
        Ok(s) => print_usage(s, json),
        Err(ApiError::NotAuthenticated) => {
            output::error("Not authenticated.");
            output::warning(format!("Run {}.", output::command("tofu login")));
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            output::error("Invalid token.");
            output::warning(format!("Run {}.", output::command("tofu login")));
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to fetch usage: {e}"));
            std::process::exit(1);
        }
    }
}
