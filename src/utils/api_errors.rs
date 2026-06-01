use crate::{
    models::api_error::ApiError,
    utils::output::{command, error, warning},
};

pub fn exit_api_error(err: ApiError, action: &str, not_found: Option<&str>) -> ! {
    match err {
        ApiError::NotAuthenticated => {
            error("Not authenticated.");
            warning(format!("Run {}.", command("tofu login")));
        }
        ApiError::UnexpectedStatus { status } if status == reqwest::StatusCode::UNAUTHORIZED => {
            error("Invalid token.");
            warning(format!("Run {} to re-authenticate.", command("tofu login")));
        }
        ApiError::UnexpectedStatus { status } if status == reqwest::StatusCode::NOT_FOUND => {
            error(not_found.unwrap_or("Resource not found."));
        }
        e => {
            error(format!("Failed to {action}: {e}"));
        }
    }

    std::process::exit(1);
}
