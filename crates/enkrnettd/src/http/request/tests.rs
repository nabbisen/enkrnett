//! ENK-SEC-017, 018, 020 at the parsing level; the socket level is covered by the integration
//! tests in `tests/limits.rs`.

use super::*;

#[test]
fn request_line_forms() {
    assert_eq!(
        parse_request_line("GET /api/1/ HTTP/1.1").unwrap(),
        ("GET".into(), "/api/1/".into(), None)
    );
    assert_eq!(
        parse_request_line("GET /x?y=1 HTTP/1.1").unwrap(),
        ("GET".into(), "/x".into(), Some("y=1".into()))
    );
    assert_eq!(
        parse_request_line("GET /x HTTP/1.0"),
        Err(ReadError::Malformed("HTTP/1.1 only"))
    );
    assert_eq!(
        parse_request_line("GET /x HTTP/2"),
        Err(ReadError::Malformed("HTTP/1.1 only"))
    );
    assert_eq!(
        parse_request_line("get /x HTTP/1.1"),
        Err(ReadError::Malformed("method"))
    );
    assert_eq!(
        parse_request_line("GET x HTTP/1.1"),
        Err(ReadError::Malformed("target"))
    );
    assert_eq!(
        parse_request_line("GET /x HTTP/1.1 extra"),
        Err(ReadError::Malformed("request line"))
    );
    assert!(parse_request_line("GET /caf\u{e9} HTTP/1.1").is_err());
}

#[test]
fn headers_are_lower_cased_and_trimmed() {
    let (line, headers) = parse_head("POST /api/1/login HTTP/1.1\r\nHost: 192.168.207.1\r\nContent-Type:  application/json \r\nEnkrnett-Session: abc").unwrap();
    assert_eq!(line, "POST /api/1/login HTTP/1.1");
    assert_eq!(
        headers[0],
        ("host".to_string(), "192.168.207.1".to_string())
    );
    assert_eq!(
        headers[1],
        ("content-type".to_string(), "application/json".to_string())
    );
    assert_eq!(
        headers[2],
        ("enkrnett-session".to_string(), "abc".to_string())
    );
}

#[test]
fn malformed_headers() {
    assert!(parse_head("GET / HTTP/1.1\r\nNoColon").is_err());
    assert!(parse_head("GET / HTTP/1.1\r\n : x").is_err());
    assert!(parse_head("GET / HTTP/1.1\r\nA: b\r\n c").is_err());
}

#[test]
fn request_header_lookup_is_case_insensitive() {
    let r = Request {
        method: "GET".into(),
        path: "/".into(),
        query: None,
        headers: vec![("enkrnett-session".into(), "t".into())],
        body: vec![],
        peer: "127.0.0.1".parse().unwrap(),
    };
    assert_eq!(r.header("Enkrnett-Session"), Some("t"));
    assert_eq!(r.header("x"), None);
}
