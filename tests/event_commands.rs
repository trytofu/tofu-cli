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

#[test]
fn events_expire_fetches_then_expires_event_payload() {
    let server = MockServer::start(vec![ok_json(&event(false)), ok_json(&event(true))]);
    let base_url = server.base_url.clone();
    let home = temp_home("events-expire");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
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

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
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
