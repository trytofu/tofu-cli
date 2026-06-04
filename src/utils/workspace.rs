use crate::{
    api::ApiClient,
    models::{api_error::ApiError, hook::Hook},
    utils::{api_errors::exit_api_error, output},
};

pub async fn resolve_workspace_id(client: &ApiClient) -> Result<Option<String>, ApiError> {
    client.me().await.map(|user| user.active_workspace_id)
}

pub async fn resolve_workspace_id_or_exit(client: &ApiClient) -> String {
    match resolve_workspace_id(client).await {
        Ok(Some(workspace_id)) => workspace_id,
        Ok(None) => exit_no_active_workspace(),
        Err(e) => exit_api_error(e, "resolve active workspace", None),
    }
}

pub async fn resolve_hook_in_workspace(
    client: &ApiClient,
    workspace_id: &str,
    slug: &str,
) -> Result<Option<Hook>, ApiError> {
    let hooks = client.list_hooks(workspace_id).await?;
    Ok(hooks.into_iter().find(|h| h.slug == slug))
}

pub async fn resolve_hook_or_exit(client: &ApiClient, slug: &str) -> Hook {
    let workspace_id = resolve_workspace_id_or_exit(client).await;
    match resolve_hook_in_workspace(client, &workspace_id, slug).await {
        Ok(Some(hook)) => hook,
        Ok(None) => exit_hook_not_found(slug),
        Err(e) => exit_api_error(
            e,
            "resolve hook",
            Some("Workspace not found or you do not have access."),
        ),
    }
}

#[allow(dead_code)]
pub async fn resolve_target_id(
    client: &ApiClient,
    hook_slug: &str,
    target_name: &str,
) -> Result<Option<String>, ApiError> {
    let Some(workspace_id) = resolve_workspace_id(client).await? else {
        return Ok(None);
    };

    let Some(hook) = resolve_hook_in_workspace(client, &workspace_id, hook_slug).await? else {
        return Ok(None);
    };

    let targets = client.list_targets(&hook.id).await?;

    Ok(targets
        .into_iter()
        .find(|t| t.name == target_name)
        .map(|t| t.id))
}

pub async fn resolve_target_id_or_exit(
    client: &ApiClient,
    hook_slug: &str,
    target_name: &str,
) -> String {
    let hook = resolve_hook_or_exit(client, hook_slug).await;

    match client.list_targets(&hook.id).await {
        Ok(targets) => targets
            .into_iter()
            .find(|target| target.name == target_name)
            .map(|target| target.id)
            .unwrap_or_else(|| exit_target_not_found(target_name, hook_slug)),
        Err(e) => exit_api_error(
            e,
            "resolve target",
            Some("Hook not found or you do not have access."),
        ),
    }
}

fn exit_target_not_found(target_name: &str, hook_slug: &str) -> ! {
    output::error(format!(
        "Target '{target_name}' not found for hook '{hook_slug}'."
    ));
    output::warning(format!(
        "Run {} to see available targets.",
        output::command(format!("tofu targets list --hook {hook_slug}"))
    ));
    std::process::exit(1);
}

fn exit_no_active_workspace() -> ! {
    output::error("No active workspace set.");
    output::warning(format!(
        "Run {} or check your access.",
        output::command("tofu workspaces use <slug>")
    ));
    std::process::exit(1);
}

fn exit_hook_not_found(slug: &str) -> ! {
    output::error(format!("Hook '{slug}' not found."));
    output::warning(format!(
        "Run {} to see available hooks.",
        output::command("tofu hooks list")
    ));
    std::process::exit(1);
}
