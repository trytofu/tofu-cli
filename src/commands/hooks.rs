use crate::{
    api::ApiClient,
    models::api_error::ApiError,
    utils::{
        api_errors::exit_api_error,
        output::{self, print_plan_limit_error},
        workspace::resolve_workspace_id_or_exit,
    },
};

pub async fn list(client: &ApiClient, json: bool) {
    let workspace_id = resolve_workspace_id_or_exit(client).await;
    match client.list_hooks(&workspace_id).await {
        Ok(hooks) => {
            if json {
                println!("{}", serde_json::json!({ "hooks": hooks }));
            } else if hooks.is_empty() {
                output::empty("No hooks found. Create one with: tofu hooks create <slug>");
            } else {
                let mut t = output::data_table(&["Slug", "Name"]);
                for h in hooks {
                    t.add_row(vec![output::cell(h.slug), output::cell(h.name)]);
                }
                println!("{t}");
            }
        }
        Err(e) => exit_api_error(
            e,
            "list hooks",
            Some("Workspace not found or you do not have access."),
        ),
    }
}

pub async fn create_hook(client: &ApiClient, slug: String, name: Option<String>, json: bool) {
    let workspace_id = resolve_workspace_id_or_exit(client).await;
    let name = name.unwrap_or_else(|| slug.clone());

    match client.create_hook(&workspace_id, name, slug).await {
        Ok(hook) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": hook.id,
                        "workspace_id": hook.workspace_id,
                        "name": hook.name,
                        "slug": hook.slug,
                        "provider_url": hook.provider_url,
                        "created_at": hook.created_at,
                        "updated_at": hook.updated_at,
                    })
                );
            } else {
                output::success(format!("Created hook: {}", hook.name));
                output::next_step(format!("Provider URL: {}", output::url(&hook.provider_url)));
            }
        }
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::CONFLICT => {
            output::error("Hook slug already exists in this workspace.");
            std::process::exit(1);
        }
        Err(ApiError::PlanLimitReached(err)) => {
            print_plan_limit_error(&err);
            std::process::exit(1);
        }
        Err(e) => exit_api_error(e, "create hook", None),
    }
}
