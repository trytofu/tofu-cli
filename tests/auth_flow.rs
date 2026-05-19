use std::{fs, process::Command};

mod support;

use support::{MockServer, config_contents, ok_json, temp_home, unauthorized, write_config};

#[test]
fn token_login_verifies_then_saves_config() {
    let user = r#"{"id":"user_1","email":"dev@example.com","created_at":"2026-01-01T00:00:00Z"}"#;
    let server = MockServer::start(vec![ok_json(user)]);
    let base_url = server.base_url.clone();
    let home = temp_home("token");

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args([
            "login",
            "--token",
            "tofu_pat_test",
            "--api-base-url",
            &base_url,
        ])
        .output()
        .expect("run tofu-cli login");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_test")
    );

    let config = config_contents(&home);
    assert!(config.contains(r#"api_base_url = ""#));
    assert!(config.contains(&base_url));
    assert!(config.contains(r#"token = "tofu_pat_test""#));

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn token_login_does_not_save_config_when_verification_fails() {
    let server = MockServer::start(vec![unauthorized()]);
    let base_url = server.base_url.clone();
    let home = temp_home("invalid-token");

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args([
            "login",
            "--token",
            "tofu_pat_bad",
            "--api-base-url",
            &base_url,
        ])
        .output()
        .expect("run tofu-cli login");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_bad")
    );

    assert!(
        !home.join(".config/tofu/config.toml").exists(),
        "config should not be saved for an invalid token"
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn device_login_polls_for_token_then_verifies_and_saves_config() {
    let start = r#"{"device_code":"device-123","user_code":"ABCD-EFGH","verification_uri":"https://trytofu.dev/device","verification_uri_complete":"https://trytofu.dev/device?code=ABCD-EFGH","expires_in":30,"interval":1}"#;
    let poll = r#"{"status":"approved","token":"tofu_pat_device"}"#;
    let user =
        r#"{"id":"user_2","email":"device@example.com","created_at":"2026-01-01T00:00:00Z"}"#;
    let server = MockServer::start(vec![ok_json(start), ok_json(poll), ok_json(user)]);
    let base_url = server.base_url.clone();
    let home = temp_home("device");

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args(["login", "--api-base-url", &base_url, "--no-browser"])
        .output()
        .expect("run tofu-cli device login");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = server.finish();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/api/device-login/start");
    assert!(requests[0].body.contains(r#""client_name":"Tofu CLI""#));
    assert_eq!(requests[1].method, "POST");
    assert_eq!(requests[1].path, "/api/device-login/poll");
    assert!(requests[1].body.contains(r#""device_code":"device-123""#));
    assert_eq!(requests[2].method, "GET");
    assert_eq!(requests[2].path, "/api/me");
    assert_eq!(
        requests[2].header("authorization"),
        Some("Bearer tofu_pat_device")
    );

    let config = config_contents(&home);
    assert!(config.contains(&base_url));
    assert!(config.contains(r#"token = "tofu_pat_device""#));

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn whoami_uses_saved_token_and_prints_json_user() {
    let user =
        r#"{"id":"user_3","email":"whoami@example.com","created_at":"2026-01-01T00:00:00Z"}"#;
    let server = MockServer::start(vec![ok_json(user)]);
    let base_url = server.base_url.clone();
    let home = temp_home("whoami");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args(["--json", "whoami"])
        .output()
        .expect("run tofu-cli whoami");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""id":"user_3""#));
    assert!(stdout.contains(r#""email":"whoami@example.com""#));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/me");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer tofu_pat_saved")
    );

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn whoami_reports_fetch_failure_without_login_wording() {
    let response =
        "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
            .to_string();
    let server = MockServer::start(vec![response]);
    let base_url = server.base_url.clone();
    let home = temp_home("whoami-failure");
    write_config(
        &home,
        &format!("api_base_url = \"{base_url}\"\ntoken = \"tofu_pat_saved\"\n"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args(["whoami"])
        .output()
        .expect("run tofu-cli whoami");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to fetch user"));
    assert!(!stderr.contains("Login failed"));

    let requests = server.finish();
    assert_eq!(requests.len(), 1);

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn logout_clears_token_but_preserves_api_base_url() {
    let home = temp_home("logout");
    write_config(
        &home,
        "api_base_url = \"http://127.0.0.1:1234\"\ntoken = \"tofu_pat_saved\"\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tofu-cli"))
        .env("HOME", &home)
        .args(["--json", "logout"])
        .output()
        .expect("run tofu-cli logout");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(r#""status":"ok""#));

    let config = config_contents(&home);
    assert!(config.contains(r#"api_base_url = "http://127.0.0.1:1234""#));
    assert!(!config.contains("token"));

    fs::remove_dir_all(home).expect("remove temp home");
}
