//! ENK-OWN-009, ENK-OWN-010.

use super::*;

fn codes(errors: &[FieldError]) -> Vec<FieldCode> {
    errors.iter().map(|e| e.code).collect()
}

#[test]
fn length_bounds() {
    let ctx = PasswordContext::default();
    assert_eq!(
        codes(&check_new_password("p", "short", &ctx)),
        vec![FieldCode::TooShort]
    );
    assert!(check_new_password("p", "exactly10!", &ctx).is_empty());
    assert!(check_new_password("p", &"x".repeat(64), &ctx).is_empty());
    assert_eq!(
        codes(&check_new_password("p", &"x".repeat(65), &ctx)),
        vec![FieldCode::TooLong]
    );
    assert_eq!(
        codes(&check_new_password("p", "", &ctx)),
        vec![FieldCode::Required]
    );
}

#[test]
fn only_printable_ascii() {
    let ctx = PasswordContext::default();
    // 9 Japanese characters are 27 bytes: long enough, but not ASCII.
    assert_eq!(
        codes(&check_new_password("p", "パスワードですよね", &ctx)),
        vec![FieldCode::ForbiddenCharacters]
    );
    assert_eq!(
        codes(&check_new_password("p", "tab\there1234", &ctx)),
        vec![FieldCode::ForbiddenCharacters]
    );
    assert!(check_new_password("p", "Spaces are fine 1!", &ctx).is_empty());
    assert!(check_new_password("p", "~!@#$%^&*()_+", &ctx).is_empty());
}

#[test]
fn common_passwords_are_refused_case_insensitively() {
    let ctx = PasswordContext::default();
    assert_eq!(
        codes(&check_new_password("p", "password123", &ctx)),
        vec![FieldCode::CommonPassword]
    );
    assert_eq!(
        codes(&check_new_password("p", "PASSWORD123", &ctx)),
        vec![FieldCode::CommonPassword]
    );
    assert!(check_new_password("p", "correct horse battery staple", &ctx).is_empty());
}

#[test]
fn equals_checks() {
    let names = ["Cafe Example", "Enkrnett-3F2A"];
    let ctx = PasswordContext {
        network_names: &names,
        management_key: Some("abcde-fghjk-mnpqr-stuvw"),
        device_id: Some("0123456789abcdef"),
    };
    assert_eq!(
        codes(&check_new_password("p", "Cafe Example", &ctx)),
        vec![FieldCode::EqualsNetworkName]
    );
    assert_eq!(
        codes(&check_new_password("p", "abcde-fghjk-mnpqr-stuvw", &ctx)),
        vec![FieldCode::EqualsKey]
    );
    assert_eq!(
        codes(&check_new_password("p", "0123456789abcdef", &ctx)),
        vec![FieldCode::EqualsDeviceId]
    );
}

#[test]
fn the_shipped_list_is_lower_case_and_within_budget() {
    assert!(COMMON.lines().all(|l| l == l.to_ascii_lowercase()));
    assert!(
        COMMON.len() < 8 * 1024,
        "the list counts against ENK-FOOT-003"
    );
}
