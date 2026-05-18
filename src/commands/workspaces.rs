use crate::{api::ApiClient, models::api_error::ApiError, utils::output};

pub async fn list(client: &ApiClient, json: bool) {
    match client.list_workspaces().await {
        Ok(workspaces) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "workspaces": workspaces,
                    })
                );
            } else if workspaces.is_empty() {
                output::empty(
                    "No workspaces found. Create one with: tofu workspaces create <name-or-slug>",
                );
            } else {
                let mut t = output::data_table(&["Slug", "Name", "Hooks"]);
                for ws in workspaces {
                    t.add_row(vec![
                        output::cell(ws.slug),
                        output::cell(ws.name),
                        output::cell(
                            ws.hook_count
                                .map(|count| count.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                    ]);
                }
                println!("{t}");
            }
        }
        Err(ApiError::NotAuthenticated) => {
            output::error("Not authenticated.");
            output::warning(format!("Run {}.", output::command("tofu login")));
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            output::error("Invalid token.");
            output::warning(format!(
                "Run {} to re-authenticate.",
                output::command("tofu login")
            ));
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to list workspaces: {e}"));
            std::process::exit(1);
        }
    }
}
