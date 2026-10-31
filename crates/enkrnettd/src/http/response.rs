//! Responses: the security headers of ENK-API-025 on every one, `Connection: close`
//! (ENK-SEC-019), `Cache-Control` per ENK-API-016 and ENK-API-024, bodies within ENK-SEC-022.

use std::io::Write;

use enkrnett_model::api::{ErrorBody, ErrorCode, ErrorDetail};
use serde::Serialize;

/// Largest response body (ENK-SEC-022).
pub const MAX_BODY: usize = 64 * 1024;

pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'";

/// Headers on every response (ENK-API-025).
pub const SECURITY_HEADERS: [(&str, &str); 6] = [
    ("Content-Security-Policy", CONTENT_SECURITY_POLICY),
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("Referrer-Policy", "no-referrer"),
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Resource-Policy", "same-origin"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Response {
        Response {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Response {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    /// A JSON body with `Cache-Control: no-store` (ENK-API-016).
    pub fn json<T: Serialize>(status: u16, value: &T) -> Response {
        let body = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
        Response::new(status)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Cache-Control", "no-store")
            .with_body(body)
    }

    /// `{}` with `no-store`.
    pub fn empty_json(status: u16) -> Response {
        Response::json(status, &serde_json::json!({}))
    }

    /// The one error shape (ENK-API-018).
    pub fn error(status: u16, code: ErrorCode, message: &str) -> Response {
        Response::json(
            status,
            &ErrorBody {
                error: ErrorDetail::new(code).message(message),
            },
        )
    }

    /// `try_later` also carries `Retry-After` (chapter 9 §5).
    pub fn error_detail(detail: ErrorDetail) -> Response {
        let status = detail.code.http_status();
        let retry_after = detail.retry_in;
        let response = Response::json(status, &ErrorBody { error: detail });
        match retry_after {
            Some(seconds) => response.header("Retry-After", &seconds.to_string()),
            None => response,
        }
    }

    /// A file body with the given cache policy.
    pub fn file(status: u16, content_type: &str, cache_control: &str, body: Vec<u8>) -> Response {
        Response::new(status)
            .header("Content-Type", content_type)
            .header("Cache-Control", cache_control)
            .with_body(body)
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Response {
        self.body = body;
        self
    }

    pub fn reason(status: u16) -> &'static str {
        match status {
            200 => "OK",
            202 => "Accepted",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            413 => "Content Too Large",
            415 => "Unsupported Media Type",
            421 => "Misdirected Request",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            500 => "Internal Server Error",
            _ => "Response",
        }
    }

    /// Serialises the response with the fixed headers added.
    pub fn to_bytes(&self) -> Vec<u8> {
        let body: &[u8] = if self.body.len() > MAX_BODY {
            crate::log::event("response_too_large", &[("bytes", &self.body.len())]);
            b"{\"error\":{\"code\":\"internal\",\"message\":\"response too large\"}}"
        } else {
            &self.body
        };
        let mut out = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status,
            Response::reason(self.status)
        )
        .into_bytes();
        for (name, value) in &self.headers {
            out.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
        }
        for (name, value) in SECURITY_HEADERS {
            out.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "Content-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        );
        out.extend_from_slice(body);
        out
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.to_bytes())?;
        w.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_response_carries_the_fixed_headers() {
        let text = String::from_utf8(Response::empty_json(200).to_bytes()).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        for (name, value) in SECURITY_HEADERS {
            assert!(text.contains(&format!("{name}: {value}\r\n")), "{name}");
        }
        assert!(text.contains("Connection: close\r\n"));
        assert!(text.contains("Cache-Control: no-store\r\n"));
        assert!(text.ends_with("\r\n\r\n{}"));
    }

    #[test]
    fn oversized_bodies_are_replaced() {
        let r = Response::new(200).with_body(vec![b'x'; MAX_BODY + 1]);
        let text = String::from_utf8(r.to_bytes()).unwrap();
        assert!(text.contains("response too large"));
    }
}
