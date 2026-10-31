//! Routing and the request conventions of chapter 9: paths matched exactly, `GET` and `POST`
//! only, a JSON body on every `POST`, strict JSON, the `Host` check, the authorization order
//! (state, then session).

pub mod json;

use std::sync::Arc;

use enkrnett_model::api::{ErrorCode, ErrorDetail, FieldCode, FieldError};
use serde::de::DeserializeOwned;

use crate::app::{ApiError, App, Reply};
use crate::config::Config;
use crate::http::{Handler, Request, Response};
use crate::static_files::StaticFiles;

pub const PREFIX: &str = "/api/1/";

pub struct Api {
    app: App,
    config: Arc<Config>,
    files: StaticFiles,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}

/// The resources of chapter 9 §3 and their methods (ENK-API-013).
const ROUTES: [(&str, Method); 22] = [
    ("", Method::Get),
    ("claim/start", Method::Post),
    ("claim", Method::Get),
    ("claim/identify", Method::Post),
    ("claim/check", Method::Post),
    ("claim/confirm", Method::Post),
    ("login", Method::Post),
    ("logout", Method::Post),
    ("status", Method::Get),
    ("settings", Method::Post),
    ("settings/restore", Method::Post),
    ("change", Method::Get),
    ("management-key", Method::Get),
    ("management-key", Method::Post),
    ("password", Method::Post),
    ("device", Method::Get),
    ("update", Method::Get),
    ("update/check", Method::Post),
    ("update/install", Method::Post),
    ("restart", Method::Post),
    ("erase", Method::Post),
    ("change", Method::Get),
];

impl Api {
    pub fn new(app: App, config: Arc<Config>) -> Api {
        let files = StaticFiles::new(config.ui_dir.clone());
        Api { app, config, files }
    }

    fn dispatch(&self, req: &Request) -> Result<Reply, ApiError> {
        let rel = &req.path[PREFIX.len()..];
        let method = match req.method.as_str() {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => {
                return Err(ApiError::code(
                    ErrorCode::MethodNotAllowed,
                    "GET and POST only",
                ));
            }
        };
        let known: Vec<Method> = ROUTES
            .iter()
            .filter(|(p, _)| *p == rel)
            .map(|(_, m)| *m)
            .collect();
        if known.is_empty() {
            return Err(ApiError::code(
                ErrorCode::UnknownResource,
                "no such resource",
            ));
        }
        if !known.contains(&method) {
            return Err(ApiError::code(
                ErrorCode::MethodNotAllowed,
                "wrong method for this resource",
            ));
        }
        if method == Method::Get && !req.body.is_empty() {
            return Err(ApiError::code(ErrorCode::Invalid, "a GET carries no body"));
        }
        let token = req.header("enkrnett-session");
        let app = &self.app;
        if method == Method::Post {
            check_content_type(req)?;
            json::check_strict(&req.body).map_err(|e| ApiError::code(ErrorCode::Invalid, e))?;
            if rel != "logout" {
                app.refuse_while_updating()?;
            }
        }
        match (rel, method) {
            ("", Method::Get) => Ok(app.greeting(req.peer)),
            ("claim/start", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.claim_start(req.peer)
            }
            ("claim", Method::Get) => app.claim_get(token, req.peer),
            ("claim/identify", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.claim_identify(token)
            }
            ("claim/check", Method::Post) => {
                let body = parse::<enkrnett_model::api::ClaimCheck>(req)?;
                app.claim_check(token, body)
            }
            ("claim/confirm", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.claim_confirm(token)
            }
            ("login", Method::Post) => {
                let body = parse::<enkrnett_model::api::LoginRequest>(req)?;
                app.login(body, req.peer)
            }
            ("logout", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.logout(token)
            }
            ("status", Method::Get) => app.status(token),
            ("settings", Method::Post) => {
                let body = parse::<enkrnett_model::settings::SettingsUpdate>(req)?;
                app.settings_post(token, body)
            }
            ("settings/restore", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.settings_restore(token)
            }
            ("change", Method::Get) => app.change_get(token),
            ("management-key", Method::Get) => app.management_key_get(token),
            ("management-key", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.management_key_post(token)
            }
            ("password", Method::Post) => {
                let body = parse::<enkrnett_model::api::PasswordChange>(req)?;
                app.password_post(token, body)
            }
            ("device", Method::Get) => app.device_get(token),
            ("update", Method::Get) => app.update_get(token),
            ("update/check", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.update_check(token)
            }
            ("update/install", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.update_install(token)
            }
            ("restart", Method::Post) => {
                parse::<enkrnett_model::api::Empty>(req)?;
                app.restart(token)
            }
            ("erase", Method::Post) => {
                let body = parse::<enkrnett_model::api::EraseRequest>(req)?;
                app.erase(token, body)
            }
            _ => Err(ApiError::code(
                ErrorCode::UnknownResource,
                "no such resource",
            )),
        }
    }
}

impl Handler for Api {
    fn handle(&self, req: &Request) -> Response {
        self.app.tick();
        match req.header("host") {
            Some(h) if self.config.allowed_host(h) => {}
            _ => return Response::new(421),
        }
        if req.path.starts_with("/api/") {
            if !req.path.starts_with(PREFIX) {
                return Response::error(404, ErrorCode::UnknownResource, "unknown API version");
            }
            return match self.dispatch(req) {
                Ok(reply) => Response::json(reply.status, &reply.body),
                Err(e) => Response::error_detail(*e.0),
            };
        }
        match req.method.as_str() {
            "GET" => self.files.get(&req.path),
            _ => Response::error(405, ErrorCode::MethodNotAllowed, "files are read with GET"),
        }
    }
}

fn check_content_type(req: &Request) -> Result<(), ApiError> {
    let ct = req.header("content-type").unwrap_or("");
    let kind = ct.split(';').next().unwrap_or("").trim();
    if !kind.eq_ignore_ascii_case("application/json") {
        return Err(ApiError::code(
            ErrorCode::WrongContentType,
            "application/json required",
        ));
    }
    Ok(())
}

/// Parses a typed request body, turning serde's messages into field errors (ENK-API-005).
fn parse<T: DeserializeOwned>(req: &Request) -> Result<T, ApiError> {
    serde_json::from_slice::<T>(&req.body).map_err(|e| {
        let text = e.to_string();
        let field_error = if let Some(name) = between(&text, "unknown field `", "`") {
            Some(FieldError::new(name, FieldCode::UnknownField))
        } else if let Some(name) = between(&text, "missing field `", "`") {
            Some(FieldError::new(name, FieldCode::Required))
        } else if text.starts_with("invalid type")
            || text.starts_with("invalid value")
            || text.starts_with("unknown variant")
        {
            Some(FieldError::new("", FieldCode::WrongType))
        } else {
            None
        };
        let mut detail = ErrorDetail::new(ErrorCode::Invalid).message(text);
        detail.fields = field_error.into_iter().collect();
        ApiError(Box::new(detail))
    })
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s = text.find(start)? + start.len();
    let e = text[s..].find(end)? + s;
    Some(&text[s..e])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_route_is_get_or_post_and_unique_per_method() {
        let mut seen = std::collections::HashSet::new();
        for (path, method) in ROUTES {
            assert!(
                seen.insert((path, method == Method::Get)) || path == "change",
                "{path}"
            );
        }
    }

    #[test]
    fn between_extracts_names() {
        assert_eq!(
            between(
                "unknown field `nmae`, expected one of",
                "unknown field `",
                "`"
            ),
            Some("nmae")
        );
        assert_eq!(
            between("missing field `password`", "missing field `", "`"),
            Some("password")
        );
        assert_eq!(between("other", "missing field `", "`"), None);
    }
}
