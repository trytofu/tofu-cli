use crate::{api::ApiClient, models::health_status::HealthStatus, utils::output};

pub async fn run(api: &ApiClient, json: bool) {
    match api.health().await {
        Ok(HealthStatus::Ok) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                output::success("Tofu is ready.");
            }
        }
        Ok(HealthStatus::NotOk(status)) => {
            output::error(format!("API returned status: {status}"));
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to connect to API: {e}"));
            std::process::exit(1);
        }
    }
}
