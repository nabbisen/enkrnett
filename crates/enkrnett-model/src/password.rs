//! Owner password rules (ENK-OWN-009, ENK-OWN-010). Hashing lives in the daemon; this module
//! only decides whether a password may be used.

use crate::api::{FieldCode, FieldError};

/// Shortest owner password (ENK-OWN-009).
pub const MIN_LEN: usize = 10;
/// Longest owner password (ENK-OWN-009).
pub const MAX_LEN: usize = 64;

/// The common-password list shipped with the release (ENK-OWN-010), one per line, lower case.
/// Its size counts against ENK-FOOT-003.
const COMMON: &str = include_str!("password/common.txt");

/// The values a new password must differ from (ENK-OWN-010). Every entry is optional because
/// during setup some do not exist yet.
#[derive(Clone, Debug, Default)]
pub struct PasswordContext<'a> {
    pub network_names: &'a [&'a str],
    pub management_key: Option<&'a str>,
    pub device_id: Option<&'a str>,
}

/// Whether every character is printable ASCII (0x20 to 0x7E), so that the same password can be
/// typed on every phone and keyboard.
pub fn is_printable_ascii(password: &str) -> bool {
    password.bytes().all(|b| (0x20..=0x7e).contains(&b))
}

/// Whether the password appears in the shipped list (case-insensitive).
pub fn is_common(password: &str) -> bool {
    let lower = password.to_ascii_lowercase();
    COMMON.lines().any(|line| line == lower)
}

/// Checks a new owner password. Returns every rule it breaks (ENK-API-019), as errors on the
/// field `field`.
pub fn check_new_password(
    field: &str,
    password: &str,
    context: &PasswordContext,
) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if password.is_empty() {
        errors.push(FieldError::new(field, FieldCode::Required));
        return errors;
    }
    if password.len() < MIN_LEN {
        errors.push(FieldError::with_limit(
            field,
            FieldCode::TooShort,
            MIN_LEN as u64,
        ));
    }
    if password.len() > MAX_LEN {
        errors.push(FieldError::with_limit(
            field,
            FieldCode::TooLong,
            MAX_LEN as u64,
        ));
    }
    if !is_printable_ascii(password) {
        errors.push(FieldError::new(field, FieldCode::ForbiddenCharacters));
    }
    if is_common(password) {
        errors.push(FieldError::new(field, FieldCode::CommonPassword));
    }
    if context
        .network_names
        .iter()
        .any(|n| !n.is_empty() && *n == password)
    {
        errors.push(FieldError::new(field, FieldCode::EqualsNetworkName));
    }
    if context.management_key.is_some_and(|k| k == password) {
        errors.push(FieldError::new(field, FieldCode::EqualsKey));
    }
    if context.device_id.is_some_and(|d| d == password) {
        errors.push(FieldError::new(field, FieldCode::EqualsDeviceId));
    }
    errors
}

#[cfg(test)]
mod tests;
