use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
struct RecordedRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl RecordedRequest {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

struct MockServer {
    base_url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    thread: thread::JoinHandle<()>,
}

impl MockServer {
    fn start(responses: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let base_url = format!("http://{}", listener.local_addr().expect("local address"));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let request_log = Arc::clone(&requests);
        let response_count = responses.len();
        let responses = Arc::new(responses);
        let response_index = Arc::new(AtomicUsize::new(0));

        let thread = thread::spawn(move || {
            for stream in listener.incoming().take(response_count) {
                let response_index = response_index.fetch_add(1, Ordering::SeqCst);
                let response = responses.get(response_index).expect("response for request");
                let mut stream = stream.expect("accept connection");
                let request = read_request(&mut stream);
                request_log.lock().expect("lock requests").push(request);
                stream
                    .write_all(response.as_bytes())
                    .expect("write mock response");
            }
        });

        Self {
            base_url,
            requests,
            thread,
        }
    }

    fn finish(self) -> Vec<RecordedRequest> {
        self.thread.join().expect("mock server thread");
        Arc::try_unwrap(self.requests)
            .expect("request log still shared")
            .into_inner()
            .expect("request log mutex")
    }
}

fn read_request(stream: &mut TcpStream) -> RecordedRequest {
    let mut buffer = Vec::new();
    let mut chunk = [0; 1024];
    let header_end;

    loop {
        let read = stream.read(&mut chunk).expect("read request");
        assert!(read > 0, "connection closed before headers");
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = find_header_end(&buffer) {
            header_end = position;
            break;
        }
    }

    let header_text = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().expect("request line");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method").to_string();
    let path = request_parts.next().expect("path").to_string();

    let headers = lines
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            Some((key.to_string(), value.trim().to_string()))
        })
        .collect::<Vec<_>>();

    let content_length = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = stream.read(&mut chunk).expect("read body");
        assert!(read > 0, "connection closed before body");
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body =
        String::from_utf8_lossy(&buffer[body_start..body_start + content_length]).to_string();

    RecordedRequest {
        method,
        path,
        headers,
        body,
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn ok_json(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn unauthorized() -> String {
    "HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string()
}

fn temp_home(test_name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("tofu-cli-{test_name}-{}-{now}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp home");
    path
}

fn config_contents(home: &PathBuf) -> String {
    fs::read_to_string(home.join(".config/tofu/config.toml")).expect("read config")
}

fn write_config(home: &PathBuf, contents: &str) {
    let config_path = home.join(".config/tofu/config.toml");
    fs::create_dir_all(config_path.parent().expect("config parent")).expect("create config parent");
    fs::write(config_path, contents).expect("write config");
}

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
