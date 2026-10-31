//! Chapter 9 conventions over a real socket: ENK-API-001 to 011, 013, 016, 018, 019, 025;
//! ENK-SEC-009, 010, 011, 012; ENK-AUTH-007.

mod common;

use common::*;

#[test]
fn greeting_has_exactly_the_specified_fields_and_headers() {
    let ts = start();
    let r = get(&ts, "", None);
    assert_eq!(r.status, 200);
    let v = r.json();
    let mut keys: Vec<&String> = v.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "api",
            "connection",
            "development",
            "languages",
            "product",
            "state"
        ]
    );
    assert_eq!(v["product"], "Enkrnett");
    assert_eq!(v["api"], 1);
    assert_eq!(v["state"], "SETUP");
    assert_eq!(v["languages"], serde_json::json!(["en", "ja"]));
    assert_eq!(v["connection"]["network"], "setup");
    assert_eq!(v["connection"]["security"], "owe");
    assert_eq!(
        r.header("Content-Type"),
        Some("application/json; charset=utf-8")
    );
    assert_eq!(r.header("Cache-Control"), Some("no-store"));
    assert_eq!(r.header("Connection"), Some("close"));
    assert_eq!(r.header("X-Frame-Options"), Some("DENY"));
    assert_eq!(r.header("X-Content-Type-Options"), Some("nosniff"));
    assert_eq!(r.header("Referrer-Policy"), Some("no-referrer"));
    assert!(
        r.header("Content-Security-Policy")
            .unwrap()
            .contains("frame-ancestors 'none'")
    );
    assert!(r.header("Access-Control-Allow-Origin").is_none());
}

#[test]
fn unknown_resources_methods_and_versions() {
    let ts = start();
    assert_eq!(get(&ts, "nope", None).code(), "unknown_resource");
    assert_eq!(get(&ts, "nope", None).status, 404);
    assert_eq!(request(ts.addr, "GET", "/api/2/", &[], None).status, 404);
    let r = request(ts.addr, "PUT", "/api/1/settings", &[], Some("{}"));
    assert_eq!((r.status, r.code()), (405, "method_not_allowed".into()));
    let r = request(
        ts.addr,
        "OPTIONS",
        "/api/1/login",
        &[("Origin", "http://evil.example")],
        None,
    );
    assert_eq!(r.status, 405);
    assert!(
        r.header("Access-Control-Allow-Origin").is_none(),
        "ENK-SEC-011"
    );
    let r = request(ts.addr, "POST", "/api/1/", &[], Some("{}"));
    assert_eq!(r.status, 405, "the greeting is read, never posted");
    let r = request(ts.addr, "GET", "/api/1/login", &[], None);
    assert_eq!(r.status, 405);
}

#[test]
fn host_header_must_name_the_device() {
    let ts = start();
    let text = raw(
        ts.addr,
        b"GET /api/1/ HTTP/1.1\r\nHost: evil.example\r\n\r\n",
    );
    let r = parse_response(&text);
    assert_eq!(r.status, 421);
    assert_eq!(r.body, "");
    let text = raw(ts.addr, b"GET /api/1/ HTTP/1.1\r\n\r\n");
    assert_eq!(parse_response(&text).status, 421, "no Host at all");
    let text = raw(
        ts.addr,
        format!(
            "GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
            ts.addr.port()
        )
        .as_bytes(),
    );
    assert_eq!(
        parse_response(&text).status,
        200,
        "the listen address with a port"
    );
    let text = raw(ts.addr, b"GET /api/1/ HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(parse_response(&text).status, 200, "a configured name");
}

#[test]
fn posts_need_a_json_body_and_gets_refuse_one() {
    let ts = start();
    let r = request(
        ts.addr,
        "POST",
        "/api/1/claim/start",
        &[("Content-Type", "text/plain"), ("Content-Length", "2")],
        None,
    );
    // the helper sends no body for None; build the raw request instead
    let _ = r;
    let text = raw(ts.addr, b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\n{}");
    let r = parse_response(&text);
    assert_eq!((r.status, r.code()), (415, "wrong_content_type".into()));
    let text = raw(
        ts.addr,
        b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\n\r\n{}",
    );
    assert_eq!(
        parse_response(&text).status,
        415,
        "a POST without Content-Type"
    );
    let text = raw(ts.addr, b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 2\r\n\r\n{}");
    assert_eq!(
        parse_response(&text).status,
        200,
        "a charset parameter is fine"
    );
    let text = raw(
        ts.addr,
        b"GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\n\r\n{}",
    );
    let r = parse_response(&text);
    assert_eq!((r.status, r.code()), (400, "invalid".into()));
}

#[test]
fn strict_json_and_field_errors() {
    let ts = start();
    let token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let r = post(&ts, "claim/check", Some(&token), r#"{"a":1,"a":2}"#);
    assert_eq!((r.status, r.code()), (400, "invalid".into()));
    assert!(
        r.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("duplicate")
    );
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        r#"{"guest":{"name":"x","mode":"open_to_all"},"owner_password":"p","extra":1.5}"#,
    );
    assert_eq!(r.status, 400);
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        r#"{"guest":{"name":"x","mode":"open_to_all"},"owner_password":"p","extra":1}"#,
    );
    assert_eq!(r.json()["error"]["fields"][0]["code"], "unknown_field");
    assert_eq!(r.json()["error"]["fields"][0]["field"], "extra");
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        r#"{"guest":{"name":"x","mode":"open_to_all"}}"#,
    );
    assert_eq!(r.json()["error"]["fields"][0]["code"], "required");
    assert_eq!(r.json()["error"]["fields"][0]["field"], "owner_password");
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        r#"{"guest":{"name":"x","mode":"sideways"},"owner_password":"p"}"#,
    );
    assert_eq!(r.json()["error"]["fields"][0]["code"], "wrong_type");
    let r = post(&ts, "claim/identify", Some(&token), r#"{"anything":1}"#);
    assert_eq!(
        r.json()["error"]["fields"][0]["code"],
        "unknown_field",
        "an action without parameters takes {{}} only"
    );
}

#[test]
fn state_is_checked_before_the_session() {
    let ts = start();
    // In SETUP, owner resources answer wrong_state with the state, with or without a token.
    for path in ["status", "device", "change", "management-key", "update"] {
        let r = get(&ts, path, None);
        assert_eq!((r.status, r.code()), (403, "wrong_state".into()), "{path}");
        assert_eq!(r.json()["error"]["state"], "SETUP");
        let r = get(&ts, path, Some("not-a-token"));
        assert_eq!(r.status, 403, "{path} with a bogus token");
    }
    for path in [
        "settings/restore",
        "management-key",
        "restart",
        "update/check",
        "update/install",
        "logout",
    ] {
        let r = post(&ts, path, None, "{}");
        assert!(
            r.status == 403 || (path == "logout" && r.status == 401),
            "{path}: {}",
            r.body
        );
    }
    let r = post(&ts, "login", None, r#"{"password":"x"}"#);
    assert_eq!(
        (r.status, r.code()),
        (403, "wrong_state".into()),
        "login is not offered in SETUP"
    );
    // Claim resources need the claim session.
    let r = get(&ts, "claim", None);
    assert_eq!((r.status, r.code()), (401, "no_session".into()));
    let r = get(&ts, "claim", Some("bogus"));
    assert_eq!((r.status, r.code()), (401, "no_session".into()));
}

#[test]
fn nothing_but_the_greeting_without_a_session_once_claimed() {
    let ts = start();
    let (token, _) = claim(&ts);
    for path in ["status", "device", "change", "management-key", "update"] {
        let r = get(&ts, path, None);
        assert_eq!((r.status, r.code()), (401, "no_session".into()), "{path}");
        assert!(
            !r.body.contains("25.12"),
            "no version without a session: {path}"
        );
    }
    assert_eq!(get(&ts, "status", Some(&token)).status, 200);
    assert_eq!(get(&ts, "", None).json()["state"], "CLAIMED");
    // Claim resources are gone.
    assert_eq!(post(&ts, "claim/start", None, "{}").code(), "wrong_state");
    assert_eq!(get(&ts, "claim", Some(&token)).code(), "wrong_state");
}

#[test]
fn files_are_served_outside_the_api() {
    let ts = start();
    let r = request(ts.addr, "GET", "/", &[], None);
    assert_eq!(r.status, 200);
    assert_eq!(r.header("Content-Type"), Some("text/html; charset=utf-8"));
    assert_eq!(r.header("Cache-Control"), Some("no-store"));
    assert!(r.header("Content-Security-Policy").is_some());
    let r = request(ts.addr, "POST", "/index.html", &[], Some("{}"));
    assert_eq!(r.status, 405);
}

#[test]
fn every_error_has_the_one_shape() {
    let ts = start();
    let errors = [
        get(&ts, "nope", None),
        get(&ts, "status", None),
        get(&ts, "claim", None),
        request(ts.addr, "PUT", "/api/1/login", &[], Some("{}")),
    ];
    for r in errors {
        let v = r.json();
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), 1, "only `error`: {}", r.body);
        assert!(v["error"]["code"].is_string());
        assert_eq!(v["error"]["code"].as_str().unwrap().to_string(), r.code());
    }
}
