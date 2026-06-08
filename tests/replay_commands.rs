use std::fs;

mod support;

use support::{MockServer, ok_json, temp_home, tofu_command, write_config};

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

fn event_list() -> &'static str {
    r#"[
        {
            "id": "event_1",
            "hook_id": "hook_1",
            "method": "POST",
            "path": "/webhooks",
            "query_string": null,
            "body_preview": "{\"ok\":true}",
            "received_at": "2026-01-01T00:00:00Z",
            "payload_expires_at": "2026-01-02T00:00:00Z",
            "metadata_expires_at": "2026-02-01T00:00:00Z",
            "payload_expired_at": null,
            "manually_expired_at": null,
            "payload_expired": false,
            "replay_available": true,
            "delivery_summary": {
                "total": 1,
                "success": 1,
                "failed": 0,
                "pending": 0
            }
        }
    ]"#
}

fn event() -> &'static str {
    r#"{
        "id": "event_1",
        "hook_id": "hook_1",
        "method": "POST",
        "path": "/webhooks",
        "query_string": null,
        "headers": {"stripe-signature":"sig"},
        "body_preview": "{\"ok\":true}",
        "received_at": "2026-01-01T00:00:00Z",
        "payload_expires_at": "2026-01-02T00:00:00Z",
        "metadata_expires_at": "2026-02-01T00:00:00Z",
        "payload_expired_at": null,
        "manually_expired_at": null,
        "payload_expired": false,
        "replay_available": true,
        "deliveries": []
    }"#
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

#[test]
fn replay_latest_resolves_hook_and_posts_event_replay() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(event_list()),
        ok_json("{}"),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("replay-latest");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = tofu_command(&home)
        .args(["--json", "replay", "latest", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli replay latest");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"accepted""#));
    assert!(stdout.contains(r#""event_id":"event_1""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/events?limit=1");
    assert_eq!(requests[3].method, "POST");
    assert_eq!(requests[3].path, "/api/events/event_1/replay");
    assert_eq!(
        requests[3].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(
        requests[3].body.is_empty(),
        "replay should not send a JSON null body"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn replay_latest_to_target_resolves_hook_event_and_target() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(event_list()),
        ok_json(targets()),
        ok_json("{}"),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("replay-latest-target");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = tofu_command(&home)
        .args([
            "--json", "replay", "latest", "--hook", "stripe", "--target", "dev",
        ])
        .output()
        .expect("run tofu-cli replay latest target");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"accepted""#));
    assert!(stdout.contains(r#""event_id":"event_1""#));
    assert!(stdout.contains(r#""target":"dev""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 5);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/events?limit=1");
    assert_eq!(requests[3].method, "GET");
    assert_eq!(requests[3].path, "/api/hooks/hook_1/targets");
    assert_eq!(requests[4].method, "POST");
    assert_eq!(requests[4].path, "/api/events/event_1/replay/target_1");
    assert_eq!(
        requests[4].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(
        requests[4].body.is_empty(),
        "targeted replay should not send a JSON null body"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn replay_event_to_target_fetches_event_then_resolves_target() {
    let server = MockServer::start(vec![ok_json(event()), ok_json(targets()), ok_json("{}")]);
    let base_url = server.base_url.clone();
    let home = temp_home("replay-target");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = tofu_command(&home)
        .args(["--json", "replay", "event_1", "--target", "dev"])
        .output()
        .expect("run tofu-cli replay target");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"accepted""#));
    assert!(stdout.contains(r#""event_id":"event_1""#));
    assert!(stdout.contains(r#""target":"dev""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/events/event_1");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/hooks/hook_1/targets");
    assert_eq!(requests[2].method, "POST");
    assert_eq!(requests[2].path, "/api/events/event_1/replay/target_1");
    assert_eq!(
        requests[2].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(
        requests[2].body.is_empty(),
        "targeted replay should not send a JSON null body"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}
