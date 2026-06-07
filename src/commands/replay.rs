use crate::{
    api::ApiClient,
    models::api_error::ApiError,
    utils::{api_errors::exit_api_error, output, workspace::resolve_hook_or_exit},
};

pub async fn run(
    client: &ApiClient,
    event_id: String,
    hook_slug: Option<String>,
    target: Option<String>,
    json: bool,
) {
    let (event_id, hook_id) = if event_id == "latest" {
        let hook_slug = match hook_slug {
            Some(h) => h,
            None => {
                output::error("--hook is required when replaying 'latest'.");
                std::process::exit(1);
            }
        };

        let hook = resolve_hook_or_exit(client, &hook_slug).await;
        let events = match client.list_events(&hook.id, 1).await {
            Ok(events) => events,
            Err(e) => exit_api_error(
                e,
                "list events",
                Some("Hook not found or you do not have access."),
            ),
        };

        if events.is_empty() {
            output::error(format!("No events found for hook '{hook_slug}'."));
            std::process::exit(1);
        }

        (events[0].id.clone(), Some(hook.id))
    } else {
        let hook_id = if target.is_some() {
            match client.get_event(&event_id).await {
                Ok(event) => Some(event.hook_id),
                Err(e) => exit_api_error(
                    e,
                    "fetch event",
                    Some("Event not found or you do not have access."),
                ),
            }
        } else {
            None
        };

        (event_id, hook_id)
    };

    if let Some(target_name) = target {
        let hook_id = match hook_id {
            Some(id) => id,
            None => {
                output::error("Could not determine hook for target resolution.");
                std::process::exit(1);
            }
        };

        let target_id = match resolve_target_id_by_hook_id(client, &hook_id, &target_name).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                output::error(format!("Target '{target_name}' not found."));
                output::warning(format!(
                    "Run {} to see available targets.",
                    output::command("tofu targets list --hook <slug>")
                ));
                std::process::exit(1);
            }
            Err(e) => exit_api_error(
                e,
                "resolve target",
                Some("Hook not found or you do not have access."),
            ),
        };

        match client.replay_event_to_target(&event_id, &target_id).await {
            Ok(()) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "accepted",
                            "event_id": event_id,
                            "target": target_name,
                        })
                    );
                } else {
                    output::success(format!(
                        "Replay started for event {event_id} to target {target_name}"
                    ));
                }
            }
            Err(e) => exit_api_error(
                e,
                "replay event",
                Some("Event or target not found or you do not have access."),
            ),
        }
    } else {
        match client.replay_event(&event_id).await {
            Ok(()) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "accepted",
                            "event_id": event_id,
                        })
                    );
                } else {
                    output::success(format!("Replay started for event {event_id}"));
                }
            }
            Err(e) => exit_api_error(
                e,
                "replay event",
                Some("Event not found or you do not have access."),
            ),
        };
    }
}

async fn resolve_target_id_by_hook_id(
    client: &ApiClient,
    hook_id: &str,
    target_name: &str,
) -> Result<Option<String>, ApiError> {
    let targets = client.list_targets(hook_id).await?;

    Ok(targets
        .into_iter()
        .find(|t| t.name == target_name)
        .map(|t| t.id))
}
