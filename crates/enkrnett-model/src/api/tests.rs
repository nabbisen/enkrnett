//! The shapes serialise as chapter 9 writes them: snake_case keys, closed enumerations, every
//! field present, `null` where allowed (ENK-API-004, ENK-API-009).

use serde_json::{Value, json};

use super::*;

#[test]
fn greeting_has_exactly_the_six_fields() {
    let g = Greeting {
        product: crate::PRODUCT,
        api: crate::API_VERSION,
        state: OwnershipState::Setup,
        languages: vec!["en".into(), "ja".into()],
        development: false,
        connection: Connection {
            network: ConnectionNetwork::Setup,
            security: Some(ConnectionSecurity::Owe),
        },
    };
    let v = serde_json::to_value(&g).unwrap();
    let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
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
    assert_eq!(v["state"], "SETUP");
    assert_eq!(v["connection"]["security"], "owe");
}

#[test]
fn connection_security_null_when_unknown() {
    let c = Connection {
        network: ConnectionNetwork::Management,
        security: None,
    };
    assert_eq!(
        serde_json::to_value(&c).unwrap(),
        json!({"network": "management", "security": null})
    );
}

#[test]
fn change_none_has_nulls_and_id_zero() {
    let v = serde_json::to_value(Change::none()).unwrap();
    assert_eq!(v["id"], 0);
    assert_eq!(v["state"], "none");
    for key in [
        "kind",
        "from_version",
        "to_version",
        "needs_confirmation",
        "confirm_in",
        "reason",
        "detail",
        "started_ago",
    ] {
        assert_eq!(v[key], Value::Null, "{key}");
    }
}

#[test]
fn error_shape_and_statuses() {
    let e = ErrorBody {
        error: ErrorDetail {
            fields: vec![FieldError::with_limit("guest.name", FieldCode::TooLong, 29)],
            ..ErrorDetail::new(ErrorCode::Invalid).message("guest.name is 31 bytes")
        },
    };
    let v = serde_json::to_value(&e).unwrap();
    assert_eq!(v["error"]["code"], "invalid");
    assert_eq!(v["error"]["fields"][0]["limit"], 29);
    assert_eq!(ErrorCode::Invalid.http_status(), 400);
    assert_eq!(ErrorCode::NoSession.http_status(), 401);
    assert_eq!(ErrorCode::WrongState.http_status(), 403);
    assert_eq!(ErrorCode::UnknownResource.http_status(), 404);
    assert_eq!(ErrorCode::MethodNotAllowed.http_status(), 405);
    assert_eq!(ErrorCode::ChangePending.http_status(), 409);
    assert_eq!(ErrorCode::TooLarge.http_status(), 413);
    assert_eq!(ErrorCode::WrongContentType.http_status(), 415);
    assert_eq!(ErrorCode::TryLater.http_status(), 429);
    assert_eq!(ErrorCode::Internal.http_status(), 500);
}

#[test]
fn requests_refuse_unknown_fields() {
    assert!(serde_json::from_str::<LoginRequest>(r#"{"password":"x","extra":1}"#).is_err());
    assert!(serde_json::from_str::<Empty>(r#"{}"#).is_ok());
    assert!(serde_json::from_str::<Empty>(r#"{"a":1}"#).is_err());
    assert!(
        serde_json::from_str::<ClaimCheck>(
            r#"{"guest":{"name":"Cafe","mode":"open_to_all"},"owner_password":"pw"}"#
        )
        .is_ok()
    );
}

#[test]
fn settings_shape_flattens_the_document_and_adds_limits() {
    let shape = SettingsShape {
        settings: Settings {
            version: 7,
            country: "JP".into(),
            guest: crate::settings::GuestSettings {
                name: "Cafe".into(),
                mode: GuestMode::OpenToAll,
                enabled: true,
            },
            management: crate::settings::ManagementSettings {
                name: "Enkrnett-3F2A".into(),
            },
        },
        limits: Limits::current(),
    };
    let v = serde_json::to_value(&shape).unwrap();
    assert_eq!(v["version"], 7);
    assert_eq!(v["limits"]["guest_name_max_bytes"]["open_to_all"], 29);
}
