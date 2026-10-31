//! ENK-SEC-017 to 020 over a real socket.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use common::*;

#[test]
fn oversized_headers_and_bodies() {
    let ts = start();
    let big = format!(
        "GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\nX-Pad: {}\r\n\r\n",
        "a".repeat(9 * 1024)
    );
    let r = parse_response(&raw(ts.addr, big.as_bytes()));
    assert_eq!((r.status, r.code()), (431, "too_large".into()));
    let big_body = format!(
        "POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        17 * 1024
    );
    let r = parse_response(&raw(ts.addr, big_body.as_bytes()));
    assert_eq!((r.status, r.code()), (413, "too_large".into()));
}

#[test]
fn only_http_1_1_without_chunking_expect_or_compression() {
    let ts = start();
    let r = parse_response(&raw(
        ts.addr,
        b"GET /api/1/ HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n",
    ));
    assert_eq!((r.status, r.code()), (400, "invalid".into()));
    let r = parse_response(&raw(ts.addr, b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\n\r\n"));
    assert_eq!(r.status, 400);
    assert!(
        r.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Transfer-Encoding")
    );
    let r = parse_response(&raw(ts.addr, b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nExpect: 100-continue\r\nContent-Length: 2\r\n\r\n{}"));
    assert_eq!(r.status, 400);
    let r = parse_response(&raw(ts.addr, b"POST /api/1/claim/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Encoding: gzip\r\nContent-Length: 2\r\n\r\n{}"));
    assert_eq!(r.status, 400);
}

#[test]
fn every_response_closes_the_connection() {
    let ts = start();
    let mut s = TcpStream::connect(ts.addr).unwrap();
    s.write_all(b"GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n")
        .unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut out = Vec::new();
    s.read_to_end(&mut out).unwrap();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("Connection: close\r\n"));
    assert!(
        s.write_all(b"GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .is_err()
            || s.read(&mut [0u8; 1]).map(|n| n == 0).unwrap_or(true)
    );
}

#[test]
fn a_slow_client_is_dropped_after_the_request_timeout() {
    let ts = start();
    let started = Instant::now();
    let mut s = TcpStream::connect(ts.addr).unwrap();
    s.write_all(b"GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\n")
        .unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut out = Vec::new();
    let _ = s.read_to_end(&mut out);
    assert!(out.is_empty(), "no response to an incomplete request");
    let elapsed = started.elapsed();
    assert!(
        elapsed >= Duration::from_millis(700) && elapsed < Duration::from_secs(4),
        "{elapsed:?}"
    );
}

#[test]
fn a_burst_of_connections_is_served() {
    let ts = start();
    let handles: Vec<_> = (0..24)
        .map(|_| {
            let addr = ts.addr;
            std::thread::spawn(move || {
                parse_response(&raw(
                    addr,
                    b"GET /api/1/ HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                ))
                .status
            })
        })
        .collect();
    let statuses: Vec<u16> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(statuses.iter().all(|s| *s == 200), "{statuses:?}");
}
