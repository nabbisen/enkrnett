//! ENK-CFG-016, ENK-CFG-018, ENK-API-019: names of 29, 30, 32, 33 bytes; Japanese names of 9 and
//! 10 characters; spaces and control characters; every failing field reported at once.

use super::*;
use crate::api::FieldCode;

fn guest(name: &str, mode: GuestMode) -> GuestSettings {
    GuestSettings {
        name: name.to_string(),
        mode,
        enabled: true,
    }
}

fn codes(errors: &[FieldError]) -> Vec<(String, FieldCode)> {
    errors.iter().map(|e| (e.field.clone(), e.code)).collect()
}

#[test]
fn limits_follow_the_modes() {
    let limits = Limits::current();
    assert_eq!(limits.guest_name_max_bytes.open_to_all, 29);
    assert_eq!(limits.guest_name_max_bytes.encrypted_only, 32);
    assert_eq!(limits.management_name_max_bytes, 32);
    assert_eq!((limits.password.min, limits.password.max), (10, 64));
}

#[test]
fn ascii_names_at_the_edges() {
    let n29 = "a".repeat(29);
    let n30 = "a".repeat(30);
    let n32 = "a".repeat(32);
    let n33 = "a".repeat(33);
    assert!(validate_guest(&guest(&n29, GuestMode::OpenToAll)).is_empty());
    assert_eq!(
        codes(&validate_guest(&guest(&n30, GuestMode::OpenToAll))),
        vec![("guest.name".into(), FieldCode::TooLong)]
    );
    assert!(validate_guest(&guest(&n32, GuestMode::EncryptedOnly)).is_empty());
    assert_eq!(
        codes(&validate_guest(&guest(&n33, GuestMode::EncryptedOnly))),
        vec![("guest.name".into(), FieldCode::TooLong)]
    );
    let too_long = validate_guest(&guest(&n30, GuestMode::OpenToAll));
    assert_eq!(too_long[0].limit, Some(29));
}

#[test]
fn japanese_names_count_three_bytes_per_character() {
    let nine = "カフェの無料ワイファ"; // 10 characters? count below
    assert_eq!(nine.chars().count(), 10);
    let ten = nine;
    let nine = &nine.chars().take(9).collect::<String>();
    assert_eq!(nine.len(), 27);
    assert_eq!(ten.len(), 30);
    assert!(validate_guest(&guest(nine, GuestMode::OpenToAll)).is_empty());
    assert_eq!(
        codes(&validate_guest(&guest(ten, GuestMode::OpenToAll))),
        vec![("guest.name".into(), FieldCode::TooLong)]
    );
    assert!(validate_guest(&guest(ten, GuestMode::EncryptedOnly)).is_empty());
}

#[test]
fn switching_to_open_to_all_with_a_long_name_is_refused_not_truncated() {
    let name = "a".repeat(31);
    let encrypted = guest(&name, GuestMode::EncryptedOnly);
    assert!(validate_guest(&encrypted).is_empty());
    let open = guest(&name, GuestMode::OpenToAll);
    let errors = validate_guest(&open);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, FieldCode::TooLong);
    assert_eq!(errors[0].limit, Some(29));
}

#[test]
fn spaces_and_control_characters() {
    assert_eq!(
        codes(&validate_name("guest.name", " cafe", 32)),
        vec![("guest.name".into(), FieldCode::ForbiddenCharacters)]
    );
    assert_eq!(
        codes(&validate_name("guest.name", "cafe ", 32)),
        vec![("guest.name".into(), FieldCode::ForbiddenCharacters)]
    );
    assert_eq!(
        codes(&validate_name("guest.name", "ca\u{0}fe", 32)),
        vec![("guest.name".into(), FieldCode::ForbiddenCharacters)]
    );
    assert_eq!(
        codes(&validate_name("guest.name", "ca\tfe", 32)),
        vec![("guest.name".into(), FieldCode::ForbiddenCharacters)]
    );
    assert!(validate_name("guest.name", "Cafe Example", 32).is_empty());
    assert_eq!(
        codes(&validate_name("guest.name", "", 32)),
        vec![("guest.name".into(), FieldCode::Required)]
    );
}

#[test]
fn every_failing_field_is_reported_at_once() {
    let settings = Settings {
        version: 1,
        country: "Japan".into(),
        guest: guest(&"x".repeat(40), GuestMode::OpenToAll),
        management: ManagementSettings {
            name: " bad".into(),
        },
    };
    let errors = validate(&settings);
    let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
    assert_eq!(fields, vec!["country", "guest.name", "management.name"]);
}

#[test]
fn an_update_keeps_country_and_version() {
    let current = Settings {
        version: 7,
        country: "JP".into(),
        guest: guest("Cafe", GuestMode::OpenToAll),
        management: ManagementSettings {
            name: "Enkrnett-3F2A".into(),
        },
    };
    let update = SettingsUpdate {
        version: 7,
        guest: guest("Cafe Example", GuestMode::EncryptedOnly),
        management: ManagementSettings {
            name: "Enkrnett-3F2A".into(),
        },
    };
    let next = apply_update(&current, &update).expect("valid");
    assert_eq!(next.version, 7);
    assert_eq!(next.country, "JP");
    assert_eq!(next.guest.name, "Cafe Example");
    assert_eq!(next.guest.mode, GuestMode::EncryptedOnly);
}

#[test]
fn unknown_fields_are_refused_by_the_shapes() {
    let json = r#"{"version":1,"guest":{"name":"a","mode":"open_to_all","enabled":true,"extra":1},"management":{"name":"m"}}"#;
    assert!(serde_json::from_str::<SettingsUpdate>(json).is_err());
    let json = r#"{"version":1,"guest":{"name":"a","mode":"open_to_all","enabled":true},"management":{"name":"m"}}"#;
    assert!(serde_json::from_str::<SettingsUpdate>(json).is_ok());
}
