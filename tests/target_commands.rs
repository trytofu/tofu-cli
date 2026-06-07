use std::{fs, process::Command};

mod support;

use support::{MockServer, ok_json, temp_home, write_config};

fn user_with_active_workspace() -> &'static str {
    r#"{
        "id": "user_1",
        "email": "dev@example.com",
        "created_at": "2026-01-01T00:00:00Z",
        "active_workspace_id": "workspace_1"
    }"#
}

fn hooks() -> &'static str {
    r#"[
        {
            "id": "hook_1",
            "workspace_id": "workspace_1",
            "name": "Stripe",
            "slug": "stripe",
            "provider_url": "https://hooks.trytofu.dev/e/hook_token",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    ]"#
}

fn targets() -> &'static str {
    r#"[
        {
            "id": "target_1",
            "hook_id": "hook_1",
            "name": "dev",
            "url": "https://dev.example.com/webhooks",
            "enabled": true,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    ]"#
}

fn target() -> &'static str {
    r#"{
        "id": "target_1",
        "hook_id": "hook_1",
        "name": "dev",
        "url": "https://dev.example.com/webhooks",
        "enabled": false,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-02T00:00:00Z"
    }"#
}

#[test]
fn targets_list_uses_active_workspace_and_hook_slug() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(targets()),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("targets-list");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "targets", "list", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli targets list");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""targets":"#));
    assert!(stdout.contains(r#""name":"dev""#));
    assert!(stdout.contains(r#""enabled":true"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(
        requests[1].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/targets");
    assert_eq!(
        requests[2].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn targets_disable_resolves_target_id_before_toggling() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(targets()),
        ok_json(target()),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("targets-disable");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "targets", "disable", "dev", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli targets disable");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"ok""#));
    assert!(stdout.contains(r#""id":"target_1""#));
    assert!(stdout.contains(r#""enabled":false"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/targets");
    assert_eq!(requests[3].method, "POST");
    assert_eq!(requests[3].path, "/api/targets/target_1/disable");
    assert_eq!(
        requests[3].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(
        requests[3].body.is_empty(),
        "disable target should not send a JSON null body"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn targets_disable_reports_missing_target_without_toggling() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json("[]"),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("targets-disable-missing");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["targets", "disable", "dev", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli targets disable");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Target 'dev' not found for hook 'stripe'."));

    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/targets");

    fs::remove_dir_all(home).expect("remove temp home");
}
