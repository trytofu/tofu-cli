use crate::{
    api::ApiClient,
    models::{api_error::ApiError, target::Target},
    utils::{
        api_errors::exit_api_error,
        output::{self, print_plan_limit_error},
        workspace::{resolve_hook_or_exit, resolve_target_id_or_exit},
    },
};

pub async fn list(client: &ApiClient, hook_slug: String, json: bool) {
    let hook = resolve_hook_or_exit(client, &hook_slug).await;

    match client.list_targets(&hook.id).await {
        Ok(targets) => {
            if json {
                println!("{}", serde_json::json!({ "targets": targets }));
            } else if targets.is_empty() {
                output::empty(
                    "No targets found. Add one with: tofu targets add <name> <url> --hook <slug>",
                );
            } else {
                let mut t = output::data_table(&["Name", "Status", "URL"]);
                for target in targets {
                    let status = if target.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    t.add_row(vec![
                        output::cell(target.name),
                        output::status_cell(status),
                        output::url_cell(&target.url),
                    ]);
                }
                println!("{t}");
            }
        }
        Err(e) => exit_api_error(
            e,
            "list hook targets",
            Some("Hook not found or you do not have access."),
        ),
    }
}

pub enum TargetStatus {
    On,
    Off,
}

impl TargetStatus {
    fn action(&self) -> &'static str {
        match self {
            Self::On => "enable target",
            Self::Off => "disable target",
        }
    }

    fn past_tense(&self) -> &'static str {
        match self {
            Self::On => "Enabled",
            Self::Off => "Disabled",
        }
    }
}

enum SetResult {
    Created,
    Updated,
}

impl SetResult {
    fn status(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
        }
    }

    fn past_tense(&self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Updated => "Updated",
        }
    }
}

pub async fn toggle(
    status: TargetStatus,
    client: &ApiClient,
    name: String,
    hook_slug: String,
    json: bool,
) {
    let target_id = resolve_target_id_or_exit(client, &hook_slug, &name).await;

    let result = match status {
        TargetStatus::On => client.enable_target(&target_id).await,
        TargetStatus::Off => client.disable_target(&target_id).await,
    };

    match result {
        Ok(target) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "target": target,
                    })
                );
            } else {
                output::success(format!("{} target: {}", status.past_tense(), target.name));
            }
        }
        Err(e) => exit_api_error(
            e,
            status.action(),
            Some("Target not found or you do not have access."),
        ),
    }
}

pub async fn delete(client: &ApiClient, name: String, hook_slug: String, json: bool) {
    let target_id = resolve_target_id_or_exit(client, &hook_slug, &name).await;

    match client.delete_target(&target_id).await {
        Ok(()) => {
            if json {
                println!("{}", serde_json::json!({ "status": "ok" }));
            } else {
                output::success(format!("Deleted target: {name}"));
            }
        }
        Err(e) => exit_api_error(
            e,
            "delete target",
            Some("Target not found or you do not have access."),
        ),
    }
}

pub async fn add(client: &ApiClient, name: String, url: String, hook_slug: String, json: bool) {
    let hook = resolve_hook_or_exit(client, &hook_slug).await;

    match client.create_target(&hook.id, name, url, true).await {
        Ok(t) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "url": t.url,
                        "enabled": t.enabled,
                    })
                );
            } else {
                output::success(format!(
                    "Created target: {} -> {}",
                    t.name,
                    output::url(&t.url)
                ));
            }
        }
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::CONFLICT => {
            output::error("Target name already exists in this hook.");
            std::process::exit(1);
        }
        Err(ApiError::PlanLimitReached(err)) => {
            print_plan_limit_error(&err);
            std::process::exit(1);
        }
        Err(e) => exit_api_error(e, "create a target", None),
    }
}

pub async fn set(client: &ApiClient, name: String, url: String, hook_slug: String, json: bool) {
    let hook = resolve_hook_or_exit(client, &hook_slug).await;

    let targets = match client.list_targets(&hook.id).await {
        Ok(t) => t,
        Err(e) => exit_api_error(
            e,
            "list targets",
            Some("Hook not found or you do not have access."),
        ),
    };

    let existing = targets.into_iter().find(|t| t.name == name);

    match existing {
        Some(t) => match client
            .update_target(&t.id, None, Some(url), Some(true))
            .await
        {
            Ok(t) => print_set_result(SetResult::Updated, &t, json),
            Err(e) => exit_api_error(
                e,
                "update target",
                Some("Target not found or you do not have access."),
            ),
        },
        None => match client.create_target(&hook.id, name, url, true).await {
            Ok(t) => print_set_result(SetResult::Created, &t, json),
            Err(ApiError::UnexpectedStatus { status })
                if status == reqwest::StatusCode::CONFLICT =>
            {
                output::error("Target name already exists in this hook.");
                std::process::exit(1);
            }
            Err(ApiError::PlanLimitReached(err)) => {
                print_plan_limit_error(&err);
                std::process::exit(1);
            }
            Err(e) => exit_api_error(e, "create target", None),
        },
    }
}

fn print_set_result(result: SetResult, target: &Target, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": result.status(),
                "id": target.id,
                "name": target.name,
                "url": target.url,
                "enabled": target.enabled,
            })
        );
    } else {
        output::success(format!(
            "{} target: {} -> {}",
            result.past_tense(),
            target.name,
            output::url(&target.url)
        ));
    }
}
