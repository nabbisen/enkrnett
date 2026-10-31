//! The settings document (chapter 6 §2) and its validation (ENK-CFG-005, 016, 018).
//!
//! Names are counted in bytes of UTF-8 as received. The UI normalises to NFC before sending
//! (ENK-UI-015); the model does not normalise, so that what is stored is what was counted.

use serde::{Deserialize, Serialize};

use crate::api::{FieldCode, FieldError};
use crate::country;

/// The two guest modes (chapter 7 §1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuestMode {
    /// OWE Transition Mode: a visible open BSS and a hidden OWE BSS. The default.
    OpenToAll,
    /// OWE only, one BSS.
    EncryptedOnly,
}

impl GuestMode {
    /// The longest guest name the mode allows, in bytes (ENK-CFG-016).
    pub fn name_max_bytes(self) -> usize {
        match self {
            GuestMode::OpenToAll => 29,
            GuestMode::EncryptedOnly => 32,
        }
    }
}

/// Longest management network name in bytes (ENK-CFG-016).
pub const MANAGEMENT_NAME_MAX_BYTES: usize = 32;

/// The guest network's part of the settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestSettings {
    pub name: String,
    pub mode: GuestMode,
    pub enabled: bool,
}

/// The management network's part of the settings. The key is never here (ENK-API-014).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagementSettings {
    pub name: String,
}

/// The settings document as stored and as returned inside `status` (without `limits`, which the
/// API adds; see [`crate::api::SettingsShape`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u64,
    pub country: String,
    pub guest: GuestSettings,
    pub management: ManagementSettings,
}

/// What `POST settings` carries: the whole document except `country` (changed only by Erase)
/// and `management.key` (its own operation) (ENK-API-014).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsUpdate {
    pub version: u64,
    pub guest: GuestSettings,
    pub management: ManagementSettings,
}

/// The limits the UI shows while a name or a password is typed (RFC 016 §1; ENK-UI-015).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub guest_name_max_bytes: GuestNameLimits,
    pub management_name_max_bytes: usize,
    pub password: PasswordLimits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestNameLimits {
    pub open_to_all: usize,
    pub encrypted_only: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordLimits {
    pub min: usize,
    pub max: usize,
}

impl Limits {
    /// The limits of this release. One source for every figure the UI shows.
    pub fn current() -> Limits {
        Limits {
            guest_name_max_bytes: GuestNameLimits {
                open_to_all: GuestMode::OpenToAll.name_max_bytes(),
                encrypted_only: GuestMode::EncryptedOnly.name_max_bytes(),
            },
            management_name_max_bytes: MANAGEMENT_NAME_MAX_BYTES,
            password: PasswordLimits {
                min: crate::password::MIN_LEN,
                max: crate::password::MAX_LEN,
            },
        }
    }
}

/// Validates a network name against ENK-CFG-016: 1 to `max_bytes` bytes of UTF-8, no leading
/// or trailing space, no control characters. `field` names the field in the error.
pub fn validate_name(field: &str, name: &str, max_bytes: usize) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if name.is_empty() {
        errors.push(FieldError::new(field, FieldCode::Required));
        return errors;
    }
    if name.len() > max_bytes {
        errors.push(FieldError::with_limit(
            field,
            FieldCode::TooLong,
            max_bytes as u64,
        ));
    }
    if name.starts_with(' ') || name.ends_with(' ') || name.chars().any(char::is_control) {
        errors.push(FieldError::new(field, FieldCode::ForbiddenCharacters));
    }
    errors
}

/// Validates the guest part. A name that is too long for `open_to_all` is refused, never
/// truncated (ENK-CFG-018); the error names the mode's limit.
pub fn validate_guest(guest: &GuestSettings) -> Vec<FieldError> {
    validate_name("guest.name", &guest.name, guest.mode.name_max_bytes())
}

/// Validates the management name.
pub fn validate_management(management: &ManagementSettings) -> Vec<FieldError> {
    validate_name(
        "management.name",
        &management.name,
        MANAGEMENT_NAME_MAX_BYTES,
    )
}

/// Validates a whole document (ENK-CFG-005). Every failing field is reported (ENK-API-019).
pub fn validate(settings: &Settings) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if !country::is_valid_code(&settings.country) {
        errors.push(FieldError::new("country", FieldCode::NotInChoices));
    }
    errors.extend(validate_guest(&settings.guest));
    errors.extend(validate_management(&settings.management));
    errors
}

/// Validates an update and, when it is valid, returns the document it would produce from
/// `current`. `version` must equal the current version (ENK-API-014); a mismatch is not a field
/// error but a conflict, so the caller checks it first.
pub fn apply_update(
    current: &Settings,
    update: &SettingsUpdate,
) -> Result<Settings, Vec<FieldError>> {
    let mut errors = validate_guest(&update.guest);
    errors.extend(validate_management(&update.management));
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Settings {
        version: current.version,
        country: current.country.clone(),
        guest: update.guest.clone(),
        management: update.management.clone(),
    })
}

#[cfg(test)]
mod tests;
