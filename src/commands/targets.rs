use crate::{
    api::ApiClient,
    utils::{
        api_errors::exit_api_error,
        output,
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
