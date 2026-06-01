use crate::{api::ApiClient, utils::output};

pub async fn resolve_workspace_id(client: &ApiClient) -> Option<String> {
    match client.me().await {
        Ok(u) => u.active_workspace_id,
        Err(_) => None,
    }
}

pub async fn resolve_workspace_id_or_exit(client: &ApiClient) -> String {
    let Some(workspace_id) = resolve_workspace_id(client).await else {
        output::error("No active workspace set.");
        output::warning(format!(
            "Run {} or check your access.",
            output::command("tofu workspaces use <slug>")
        ));
        std::process::exit(1);
    };

    workspace_id
}

#[allow(dead_code)]
pub async fn resolove_workspace_id(client: &ApiClient) -> Option<String> {
    resolve_workspace_id(client).await
}
