use crate::{
    api::ApiClient,
    models::api_error::ApiError,
    utils::{
        output::{self, print_plan_limit_error},
        strings::slugify_workspace_slug,
        time::fmt_time,
        workspace::resolve_workspace_id,
    },
};

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

pub async fn cli_use(client: &ApiClient, slug: String, json: bool) {
    let workspaces = match client.list_workspaces().await {
        Ok(w) => w,
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
    };

    let normalised_slug = slugify_workspace_slug(&slug);
    let workspace = workspaces
        .iter()
        .find(|w| w.slug == slug)
        .or_else(|| workspaces.iter().find(|w| w.slug == normalised_slug));

    let Some(workspace) = workspace else {
        output::error(format!("Workspace '{slug}' not found."));
        output::warning(format!(
            "Run {} to see available workspaces.",
            output::command("tofu workspaces list")
        ));
        std::process::exit(1);
    };

    let active_slug = workspace.slug.clone();
    if let Err(e) = client.set_active_workspace(&workspace.id).await {
        output::error(format!("Failed to set active workspace: {e}"));
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "ok",
                "active_workspace": active_slug,
            })
        );
    } else {
        output::success(format!("Active workspace set to: {active_slug}"));
    }
}

pub async fn create(client: &ApiClient, slug: String, name: Option<String>, json: bool) {
    let name = name.unwrap_or_else(|| slug.clone());
    let slug = slugify_workspace_slug(&slug);

    if slug.is_empty() {
        output::error("Workspace slug cannot be empty.");
        std::process::exit(1);
    }

    match client.create_workspace(name, slug).await {
        Ok(workspace) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": workspace.id,
                        "name": workspace.name,
                        "slug": workspace.slug,
                        "created_at": workspace.created_at,
                        "updated_at": workspace.updated_at,
                    })
                );
            } else {
                output::success(format!(
                    "Created workspace: {} ({})",
                    workspace.name, workspace.slug
                ));
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
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::CONFLICT => {
            output::error("Workspace slug already exists.");
            std::process::exit(1);
        }
        Err(e) => {
            output::error(format!("Failed to create workspace: {e}"));
            std::process::exit(1);
        }
    }
}

pub async fn members_list(client: &ApiClient, json: bool) {
    let Some(workspace_id) = resolve_workspace_id(client).await else {
        eprintln!(
            "No active workspace set. Run `tofu workspaces use <slug>` or check your access."
        );
        std::process::exit(1);
    };

    match client.list_members(&workspace_id).await {
        Ok(members) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "members": members,
                    })
                );
            } else if members.is_empty() {
                output::empty(
                    "No members found. Add one with: tofu workspaces members add <email>",
                );
            } else {
                let mut t = output::data_table(&["Email", "Role", "Created"]);
                for m in members {
                    t.add_row(vec![
                        output::cell(m.email),
                        output::cell(m.role),
                        output::cell(fmt_time(&m.created_at)),
                    ]);
                }
                println!("{t}");
            }
        }
        Err(ApiError::NotAuthenticated) => {
            eprintln!("Not authenticated. Run `tofu login`");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            eprintln!("Invalid token. Run `tofu login` to re-authenticate.");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::NOT_FOUND => {
            eprintln!("Workspace not found or you do not have access.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to list members: {e}");
            std::process::exit(1);
        }
    }
}

pub async fn members_add(client: &ApiClient, email: String, json: bool) {
    let Some(workspace_id) = resolve_workspace_id(client).await else {
        eprintln!(
            "No active workspace set. Run `tofu workspaces use <slug>` or check your access."
        );
        std::process::exit(1);
    };

    match client.add_member(&workspace_id, email.clone()).await {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "email": email,
                    })
                );
            } else {
                output::success(format!("Added member: {email}"));
            }
        }
        Err(ApiError::NotAuthenticated) => {
            eprintln!("Not authenticated. Run `tofu login`");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status })
            if status == reqwest::StatusCode::UNAUTHORIZED =>
        {
            eprintln!("Invalid token. Run `tofu login` to re-authenticate.");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::NOT_FOUND => {
            eprintln!("Workspace not found or user does not exist.");
            std::process::exit(1);
        }
        Err(ApiError::UnexpectedStatus { status }) if status == reqwest::StatusCode::CONFLICT => {
            eprintln!("User is already a member of this workspace.");
            std::process::exit(1);
        }
        Err(ApiError::PlanLimitReached(err)) => {
            print_plan_limit_error(&err);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to add member: {e}");
            std::process::exit(1);
        }
    }
}
