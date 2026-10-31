//! A daemon on an ephemeral port against the mock platform, and a small HTTP/1.1 client that
//! speaks exactly what the daemon expects. Shared by the contract tests.

#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use enkrnettd::app::App;
use enkrnettd::clock::MockClock;
use enkrnettd::config::Config;
use enkrnettd::platform::MockPlatform;
use serde_json::Value;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct TestServer {
    pub addr: SocketAddr,
    pub app: App,
    pub platform: Arc<MockPlatform>,
    pub clock: MockClock,
    dir: PathBuf,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub fn config_for_tests(dir: &Path) -> Config {
    Config {
        listen: "127.0.0.1:0".parse().unwrap(),
        state_dir: dir.to_path_buf(),
        ui_dir: None,
        development: false,
        act_delay: Duration::from_millis(0),
        hosts: vec!["localhost".into()],
        request_timeout: Duration::from_millis(800),
        response_timeout: Duration::from_secs(5),
        version: "0.1.0".into(),
        languages: vec!["en".into(), "ja".into()],
        // Small argon2 parameters keep the tests fast; the device uses the profile's.
        argon2: (64, 1, 1),
    }
}

pub fn start() -> TestServer {
    start_with(|_| {})
}

pub fn start_with(setup: impl FnOnce(&MockPlatform)) -> TestServer {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("enkrnettd-it-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let platform = Arc::new(MockPlatform::new());
    setup(&platform);
    let clock = MockClock::new();
    let running = enkrnettd::start(
        config_for_tests(&dir),
        platform.clone(),
        Arc::new(clock.clone()),
    )
    .expect("start");
    TestServer {
        addr: running.addr,
        app: running.app,
        platform,
        clock,
        dir,
    }
}

pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Resp {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|e| panic!("not JSON ({e}): {}", self.body))
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(n, _)| n.to_ascii_lowercase() == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn code(&self) -> String {
        self.json()["error"]["code"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }
}

/// Sends raw bytes and reads the whole response.
pub fn raw(addr: SocketAddr, bytes: &[u8]) -> String {
    let mut s = TcpStream::connect(addr).expect("connect");
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(bytes).expect("write");
    let mut out = Vec::new();
    let _ = s.read_to_end(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

pub fn parse_response(text: &str) -> Resp {
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text, ""));
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(n, v)| (n.trim().to_string(), v.trim().to_string()))
        .collect();
    Resp {
        status,
        headers,
        body: body.to_string(),
    }
}

pub fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Resp {
    let mut text = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n");
    for (n, v) in headers {
        text.push_str(&format!("{n}: {v}\r\n"));
    }
    if let Some(b) = body {
        text.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            b.len()
        ));
    }
    text.push_str("\r\n");
    if let Some(b) = body {
        text.push_str(b);
    }
    parse_response(&raw(addr, text.as_bytes()))
}

pub fn get(ts: &TestServer, path: &str, token: Option<&str>) -> Resp {
    let headers: Vec<(&str, &str)> = token
        .map(|t| vec![("Enkrnett-Session", t)])
        .unwrap_or_default();
    request(ts.addr, "GET", &format!("/api/1/{path}"), &headers, None)
}

pub fn post(ts: &TestServer, path: &str, token: Option<&str>, body: &str) -> Resp {
    let headers: Vec<(&str, &str)> = token
        .map(|t| vec![("Enkrnett-Session", t)])
        .unwrap_or_default();
    request(
        ts.addr,
        "POST",
        &format!("/api/1/{path}"),
        &headers,
        Some(body),
    )
}

pub const PASSWORD: &str = "correct horse battery staple";

/// Walks the claim to `CLAIMED` and returns the owner session token and the proposal.
pub fn claim(ts: &TestServer) -> (String, Value) {
    let session = post(ts, "claim/start", None, "{}");
    assert_eq!(session.status, 200, "{}", session.body);
    let claim_token = session.json()["token"].as_str().unwrap().to_string();
    let check = post(
        ts,
        "claim/check",
        Some(&claim_token),
        &format!(
            r#"{{"country":"JP","guest":{{"name":"Cafe Example","mode":"open_to_all"}},"owner_password":"{PASSWORD}"}}"#
        ),
    );
    assert_eq!(check.status, 200, "{}", check.body);
    let proposal = check.json();
    let confirm = post(ts, "claim/confirm", Some(&claim_token), "{}");
    assert_eq!(confirm.status, 200, "{}", confirm.body);
    wait_for_state(ts, "PENDING");
    let login = post(
        ts,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!(login.status, 200, "{}", login.body);
    let token = login.json()["token"].as_str().unwrap().to_string();
    assert_eq!(login.json()["state"], "CLAIMED");
    (token, proposal)
}

pub fn wait_for_state(ts: &TestServer, state: &str) {
    for _ in 0..100 {
        if get(ts, "", None).json()["state"] == state {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("state did not become {state}: {}", get(ts, "", None).body);
}
