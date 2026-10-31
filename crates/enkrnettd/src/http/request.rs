//! Reading and parsing one request within the limits of ENK-SEC-017, 018, 020.

use std::io::Read;
use std::net::{IpAddr, TcpStream};
use std::time::{Duration, Instant};

/// Request line and headers together (ENK-SEC-017).
pub const MAX_HEAD: usize = 8 * 1024;
/// Request body (ENK-SEC-017).
pub const MAX_BODY: usize = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    /// The path without the query string.
    pub path: String,
    pub query: Option<String>,
    /// Header names lower-cased; values trimmed.
    pub headers: Headers,
    pub body: Vec<u8>,
    pub peer: IpAddr,
}

impl Request {
    /// The first header with that (case-insensitive) name.
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    /// Nothing more to do: the client went away or did not deliver in time.
    ClosedOrTimedOut,
    HeadersTooLarge,
    BodyTooLarge,
    Malformed(&'static str),
}

/// Reads one request. The whole request must arrive within `timeout` (ENK-SEC-018).
pub fn read_request(
    stream: &mut TcpStream,
    peer: IpAddr,
    timeout: Duration,
) -> Result<Request, ReadError> {
    let deadline = Instant::now() + timeout;
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    let head_end = loop {
        if let Some(pos) = find_head_end(&buf) {
            break pos;
        }
        if buf.len() >= MAX_HEAD {
            return Err(ReadError::HeadersTooLarge);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ReadError::ClosedOrTimedOut);
        }
        let _ = stream.set_read_timeout(Some(remaining));
        let mut chunk = [0u8; 1024];
        match stream.read(&mut chunk) {
            Ok(0) => return Err(ReadError::ClosedOrTimedOut),
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(_) => return Err(ReadError::ClosedOrTimedOut),
        }
    };
    if head_end > MAX_HEAD {
        return Err(ReadError::HeadersTooLarge);
    }
    let head = std::str::from_utf8(&buf[..head_end])
        .map_err(|_| ReadError::Malformed("head is not ASCII"))?;
    let (request_line, headers) = parse_head(head)?;
    let (method, path, query) = parse_request_line(request_line)?;

    for (name, _) in &headers {
        match name.as_str() {
            "transfer-encoding" => {
                return Err(ReadError::Malformed("Transfer-Encoding is not accepted"));
            }
            "expect" => return Err(ReadError::Malformed("Expect is not accepted")),
            "content-encoding" => {
                return Err(ReadError::Malformed("Content-Encoding is not accepted"));
            }
            _ => {}
        }
    }
    let content_length = match headers.iter().find(|(n, _)| n == "content-length") {
        Some((_, v)) => v
            .parse::<usize>()
            .map_err(|_| ReadError::Malformed("Content-Length is not a number"))?,
        None => 0,
    };
    if content_length > MAX_BODY {
        return Err(ReadError::BodyTooLarge);
    }
    let body_start = head_end + 4;
    let mut body: Vec<u8> = buf[body_start.min(buf.len())..].to_vec();
    if body.len() > content_length {
        return Err(ReadError::Malformed("more bytes than Content-Length"));
    }
    while body.len() < content_length {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ReadError::ClosedOrTimedOut);
        }
        let _ = stream.set_read_timeout(Some(remaining));
        let mut chunk = vec![0u8; (content_length - body.len()).min(4096)];
        match stream.read(&mut chunk) {
            Ok(0) => return Err(ReadError::ClosedOrTimedOut),
            Ok(n) => body.extend_from_slice(&chunk[..n]),
            Err(_) => return Err(ReadError::ClosedOrTimedOut),
        }
    }
    Ok(Request {
        method,
        path,
        query,
        headers,
        body,
        peer,
    })
}

fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

/// Header names lower-cased, values trimmed.
pub type Headers = Vec<(String, String)>;

/// Splits the head into the request line and the headers.
pub fn parse_head(head: &str) -> Result<(&str, Headers), ReadError> {
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or(ReadError::Malformed("empty request"))?;
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            return Err(ReadError::Malformed("header folding is not accepted"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(ReadError::Malformed("header without colon"))?;
        if name.is_empty() || name.contains(' ') {
            return Err(ReadError::Malformed("header name"));
        }
        headers.push((name.to_ascii_lowercase(), value.trim().to_string()));
    }
    Ok((request_line, headers))
}

/// `METHOD SP target SP HTTP/1.1` (ENK-SEC-020: HTTP/1.1 only).
pub fn parse_request_line(line: &str) -> Result<(String, String, Option<String>), ReadError> {
    let mut parts = line.split(' ');
    let method = parts
        .next()
        .filter(|m| !m.is_empty())
        .ok_or(ReadError::Malformed("method"))?;
    let target = parts.next().ok_or(ReadError::Malformed("target"))?;
    let version = parts.next().ok_or(ReadError::Malformed("version"))?;
    if parts.next().is_some() {
        return Err(ReadError::Malformed("request line"));
    }
    if version != "HTTP/1.1" {
        return Err(ReadError::Malformed("HTTP/1.1 only"));
    }
    if !method.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(ReadError::Malformed("method"));
    }
    if !target.starts_with('/') || target.bytes().any(|b| b <= b' ' || b >= 0x7f) {
        return Err(ReadError::Malformed("target"));
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (target.to_string(), None),
    };
    Ok((method.to_string(), path, query))
}

#[cfg(test)]
mod tests;
