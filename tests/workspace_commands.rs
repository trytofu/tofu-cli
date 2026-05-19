use std::{fs, process::Command};

mod support;

use support::{MockServer, ok_json, temp_home, write_config};

#[test]
fn workspaces_list_json_uses_saved_token() {
    let workspaces = r#"[
        {
            "id": "workspace_1",
            "name": "Acme Dev",
            "slug": "acme-dev",
            "hook_count": 3,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z"
        },
        {
            "id": "workspace_2",
            "name": "Sandbox",
            "slug": "sandbox",
            "hook_count": null,
            "created_at": "2026-01-03T00:00:00Z",
            "updated_at": "2026-01-04T00:00:00Z"
        }
    ]"#;
    let server = MockServer::start(vec![ok_json(workspaces)]);
    let base_url = server.base_url.clone();
    let home = temp_home("workspaces-list");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args(["--json", "workspaces", "list"])
        .output()
        .expect("run tofu-cli workspaces list");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""workspaces":"#));
    assert!(stdout.contains(r#""slug":"acme-dev""#));
    assert!(stdout.contains(r#""name":"Sandbox""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/workspaces");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn workspaces_create_posts_normalized_slug_and_prints_json() {
    let workspace = r#"{
        "id": "workspace_3",
        "name": "Stripe Team",
        "slug": "stripe-team",
        "hook_count": 0,
        "created_at": "2026-01-05T00:00:00Z",
        "updated_at": "2026-01-05T00:00:00Z"
    }"#;
    let server = MockServer::start(vec![ok_json(workspace)]);
    let base_url = server.base_url.clone();
    let home = temp_home("workspaces-create");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args([
            "--json",
            "workspaces",
            "create",
            "Stripe   Team",
            "--name",
            "Stripe Team",
        ])
        .output()
        .expect("run tofu-cli workspaces create");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""id":"workspace_3""#));
    assert!(stdout.contains(r#""name":"Stripe Team""#));
    assert!(stdout.contains(r#""slug":"stripe-team""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/api/workspaces");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(requests[0].body.contains(r#""name":"Stripe Team""#));
    assert!(requests[0].body.contains(r#""slug":"stripe-team""#));
    assert!(
        !requests[0].body.contains("stripe---team"),
        "slug should collapse repeated separators"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}
