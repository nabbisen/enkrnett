//! The UI's static files (chapter 3; ENK-API-024, ENK-API-026): the entry page `no-store`,
//! content-hashed files immutable, anything that is not a file falls back to the entry page.

use std::path::{Component, Path, PathBuf};

use enkrnett_model::api::ErrorCode;

use crate::http::Response;

/// Served when no UI directory is configured (development and the harness before `ui/` exists).
const PLACEHOLDER: &str = "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Enkrnett</title></head>\
<body><h1>Enkrnett</h1><p>The management UI is not installed on this build. The API answers under <code>/api/1/</code>.</p></body></html>\n";

pub struct StaticFiles {
    root: Option<PathBuf>,
}

impl StaticFiles {
    pub fn new(root: Option<PathBuf>) -> StaticFiles {
        StaticFiles { root }
    }

    /// Answers a `GET` outside `/api/`.
    pub fn get(&self, path: &str) -> Response {
        let Some(root) = &self.root else {
            return Response::file(
                200,
                "text/html; charset=utf-8",
                "no-store",
                PLACEHOLDER.as_bytes().to_vec(),
            );
        };
        let relative = match sanitize(path) {
            Some(r) => r,
            None => return Response::error(404, ErrorCode::UnknownResource, "path"),
        };
        let candidate = if relative.as_os_str().is_empty() {
            root.join("index.html")
        } else {
            root.join(&relative)
        };
        if candidate.is_file() {
            return serve_file(
                &candidate,
                relative
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("index.html"),
            );
        }
        if has_asset_extension(&relative) {
            return Response::error(404, ErrorCode::UnknownResource, "no such file");
        }
        serve_file(&root.join("index.html"), "index.html")
    }
}

fn serve_file(path: &Path, name: &str) -> Response {
    match std::fs::read(path) {
        Ok(body) => {
            let cache = if name == "index.html" {
                "no-store"
            } else if is_content_hashed(name) {
                "max-age=31536000, immutable"
            } else {
                "no-cache"
            };
            Response::file(200, content_type(name), cache, body)
        }
        Err(_) => Response::error(404, ErrorCode::UnknownResource, "no entry page"),
    }
}

/// Decodes `%XX`, refuses `..`, absolute components, and NUL.
pub fn sanitize(path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(path)?;
    if decoded.contains('\0') {
        return None;
    }
    let mut out = PathBuf::new();
    for component in Path::new(decoded.trim_start_matches('/')).components() {
        match component {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let v = u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
            out.push(v);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// `name-0123abcd.js`, `name.0123abcd.css`: a run of at least eight hex digits before the
/// extension, separated by `-` or `.`.
pub fn is_content_hashed(name: &str) -> bool {
    let stem = match name.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => return false,
    };
    stem.rsplit(['-', '.'])
        .next()
        .is_some_and(|run| run.len() >= 8 && run.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn has_asset_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| !e.is_empty())
}

pub fn content_type(name: &str) -> &'static str {
    match name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("json") | Some("map") => "application/json; charset=utf-8",
        Some("webmanifest") => "application/manifest+json",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests;
