#![allow(dead_code)]

use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl RecordedRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

pub struct MockServer {
    pub base_url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    thread: thread::JoinHandle<()>,
}

impl MockServer {
    pub fn start(responses: Vec<String>) -> Self {
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

    pub fn finish(self) -> Vec<RecordedRequest> {
        self.thread.join().expect("mock server thread");
        Arc::try_unwrap(self.requests)
            .expect("request log still shared")
            .into_inner()
            .expect("request log mutex")
    }
}

pub fn ok_json(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

pub fn unauthorized() -> String {
    "HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string()
}

pub fn temp_home(test_name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("tofu-cli-{test_name}-{}-{now}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp home");
    path
}

pub fn tofu_command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tofu"));
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("TOFU_CONFIG_PATH", config_path(home));
    command
}

pub fn config_contents(home: &Path) -> String {
    fs::read_to_string(config_path(home)).expect("read config")
}

pub fn write_config(home: &Path, contents: &str) {
    let config_path = config_path(home);
    fs::create_dir_all(config_path.parent().expect("config parent")).expect("create config parent");
    fs::write(config_path, contents).expect("write config");
}

fn config_path(home: &Path) -> PathBuf {
    home.join(".config").join("tofu").join("config.toml")
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
