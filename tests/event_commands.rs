use std::{fs, process::Command};

mod support;

use support::{MockServer, ok_json, temp_home, write_config};

fn event(payload_expired: bool) -> String {
    let headers = if payload_expired {
        "null"
    } else {
        r#"{"stripe-signature":"sig"}"#
    };
    let body_preview = if payload_expired {
        "null".to_string()
    } else {
        r#""{\"ok\":true}""#.to_string()
    };
    let payload_expired_at = if payload_expired {
        r#""2026-01-01T00:05:00Z""#
    } else {
        "null"
    };
    let manually_expired_at = payload_expired_at;

    format!(
        r#"{{
            "id": "event_1",
            "hook_id": "hook_1",
            "method": "POST",
            "path": "/webhooks",
            "query_string": null,
            "headers": {headers},
            "body_preview": {body_preview},
            "received_at": "2026-01-01T00:00:00Z",
            "payload_expires_at": "2026-01-02T00:00:00Z",
            "metadata_expires_at": "2026-02-01T00:00:00Z",
            "payload_expired_at": {payload_expired_at},
            "manually_expired_at": {manually_expired_at},
            "payload_expired": {payload_expired},
            "replay_available": false,
            "deliveries": []
        }}"#
    )
}

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

#[test]
fn events_list_uses_active_workspace_hook_and_limit() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(event_list()),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-list");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args([
            "--json", "events", "list", "--hook", "stripe", "--limit", "5",
        ])
        .output()
        .expect("run tofu-cli events list");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""events":"#));
    assert!(stdout.contains(r#""id":"event_1""#));
    assert!(stdout.contains(r#""replay_available":true"#));

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
    assert_eq!(requests[2].path, "/api/hooks/hook_1/events?limit=5");
    assert_eq!(
        requests[2].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn events_show_fetches_event_detail() {
    let server = MockServer::start(vec![ok_json(&event(false))]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-show");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "events", "show", "event_1"])
        .output()
        .expect("run tofu-cli events show");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""id":"event_1""#));
    assert!(stdout.contains(r#""headers":{"stripe-signature":"sig"}"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/events/event_1");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn events_latest_lists_one_then_fetches_event_detail() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json(event_list()),
        ok_json(&event(false)),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-latest");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "events", "latest", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli events latest");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""id":"event_1""#));
    assert!(stdout.contains(r#""deliveries":[]"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/events?limit=1");
    assert_eq!(requests[3].method, "GET");
    assert_eq!(requests[3].path, "/api/events/event_1");
    assert_eq!(
        requests[3].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn events_latest_prints_null_when_no_events_exist() {
    let server = MockServer::start(vec![
        ok_json(user_with_active_workspace()),
        ok_json(hooks()),
        ok_json("[]"),
    ]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-latest-empty");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "events", "latest", "--hook", "stripe"])
        .output()
        .expect("run tofu-cli events latest");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(r#""event":null"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/workspaces/workspace_1/hooks");
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/hooks/hook_1/events?limit=1");

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn events_expire_fetches_then_expires_event_payload() {
    let server = MockServer::start(vec![ok_json(&event(false)), ok_json(&event(true))]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-expire");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "events", "expire", "event_1"])
        .output()
        .expect("run tofu-cli events expire");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"ok""#));
    assert!(stdout.contains(r#""already_expired":false"#));
    assert!(stdout.contains(r#""payload_expired":true"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/events/event_1");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert_eq!(requests[1].method, "POST");
    assert_eq!(requests[1].path, "/api/events/event_1/expire");
    assert_eq!(
        requests[1].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );
    assert!(
        requests[1].body.is_empty(),
        "expire event should not send a JSON null body"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn events_expire_skips_expire_call_when_payload_already_expired() {
    let server = MockServer::start(vec![ok_json(&event(true))]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-expire-already-expired");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu"))
        .env("HOME", &home)
        .args(["--json", "events", "expire", "event_1"])
        .output()
        .expect("run tofu-cli events expire");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""status":"ok""#));
    assert!(stdout.contains(r#""already_expired":true"#));
    assert!(stdout.contains(r#""payload_expired":true"#));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/events/event_1");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}
