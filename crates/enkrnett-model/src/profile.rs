//! Device profiles (chapter 5; ENK-TIER-001, 002): the schema of `openwrt/profiles/*/profile.json`
//! and the rules a profile must satisfy. Parsing only; the files are read by the build and by
//! the daemon's platform.

use serde::{Deserialize, Serialize};

pub const PROFILE_FORMAT: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceClass {
    Standard,
    Constrained,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallClass {
    SelfInstall,
    ExpertInstall,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Install {
    pub class: InstallClass,
    pub vendor_image_kind: String,
    #[serde(default)]
    pub tested_vendor_firmware: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Light {
    pub led: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadioSpec {
    pub band: String,
    pub chip: String,
    pub interface_limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelAddress {
    pub printed: String,
    pub source: String,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Country {
    pub fixed: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Argon2Params {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

/// What the release index states for this device (ENK-RLS-006): the free memory an installation
/// needs beyond the image itself (ENK-UPD-003) and the duration the confirmation screen names
/// (RFC 017 §5.6). The release tool copies them into the index entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateFigures {
    pub min_free_kib: u64,
    pub install_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSpec {
    #[serde(default)]
    pub packages_remove: Vec<String>,
    #[serde(default)]
    pub packages_add: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verified {
    pub on_hardware: bool,
    #[serde(default)]
    pub by: Vec<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

/// One device profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfile {
    pub format: u32,
    pub id: String,
    /// `board_name` values of `ubus call system board` that select this profile.
    #[serde(default)]
    pub board_names: Vec<String>,
    pub vendor: String,
    pub model: String,
    pub revision: String,
    pub tier: u8,
    pub device_class: DeviceClass,
    pub target: String,
    pub package_arch: String,
    pub image_builder_profile: String,
    pub install: Install,
    pub flash_image_limit_kib: u32,
    pub ram_mib: u32,
    pub update: UpdateFigures,
    pub uplink_port: String,
    pub reset_button: String,
    pub light: Option<Light>,
    pub radios: Vec<RadioSpec>,
    pub platform_hardening_complete: bool,
    pub label_address: LabelAddress,
    pub country: Country,
    pub argon2: Argon2Params,
    pub image: ImageSpec,
    pub verified: Verified,
}

/// A rule of chapter 5 that a profile breaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileError(pub String);

impl DeviceProfile {
    pub fn parse(json: &str) -> Result<DeviceProfile, String> {
        let profile: DeviceProfile = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if profile.format != PROFILE_FORMAT {
            return Err(format!(
                "profile format {} is not {PROFILE_FORMAT}",
                profile.format
            ));
        }
        Ok(profile)
    }

    /// The rules every profile must meet.
    pub fn check(&self) -> Vec<ProfileError> {
        let mut errors = Vec::new();
        let err = |s: String| ProfileError(s);
        if !(1..=3).contains(&self.tier) {
            errors.push(err(format!("tier {} is not 1, 2, or 3", self.tier)));
        }
        if self.tier == 1 && !self.verified.on_hardware {
            errors.push(err(
                "Tier 1 needs verification on hardware (ENK-TIER-004)".into()
            ));
        }
        if self.tier == 1 && !self.platform_hardening_complete {
            errors.push(err(
                "Tier 1 on reduced hardening needs a separate owner decision (ENK-TIER-005)".into(),
            ));
        }
        if self.radios.is_empty() {
            errors.push(err("at least one radio".into()));
        }
        if !self.radios.iter().any(|r| r.interface_limit >= 4) {
            errors.push(err(
                "no radio allows 4 interfaces (chapter 5 §2 criterion 5)".into(),
            ));
        }
        if self.device_class == DeviceClass::Constrained && self.flash_image_limit_kib > 8 * 1024 {
            errors.push(err("a constrained device has about 8 MB of flash".into()));
        }
        if self.device_class == DeviceClass::Standard && self.ram_mib < 128 {
            errors.push(err("a standard device has 128 MB of RAM or more".into()));
        }
        if let Some(c) = &self.country.fixed {
            if !crate::country::is_valid_code(c) {
                errors.push(err(format!("country {c} is not a two-letter code")));
            }
        }
        if self.update.min_free_kib < 8 * 1024
            || self.update.min_free_kib >= u64::from(self.ram_mib) * 1024
        {
            errors.push(err(
                "update.min_free_kib must be at least 8 MiB and below the device's RAM".into(),
            ));
        }
        if !(30..=3600).contains(&self.update.install_seconds) {
            errors.push(err(
                "update.install_seconds must be between 30 and 3600".into()
            ));
        }
        if self.argon2.memory_kib == 0
            || self.argon2.iterations == 0
            || self.argon2.parallelism == 0
        {
            errors.push(err("argon2 parameters must be positive".into()));
        }
        if self.board_names.is_empty() {
            errors.push(err(
                "board_names must name the board(s) this profile is for".into(),
            ));
        }
        if self.label_address.printed.is_empty() || self.label_address.source.is_empty() {
            errors.push(err(
                "label_address must say what the label prints and where it is read".into(),
            ));
        }
        if !self
            .id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            errors.push(err(
                "id uses lower-case letters, digits, '_' and '-' only".into()
            ));
        }
        errors
    }
}

#[cfg(test)]
mod tests;
