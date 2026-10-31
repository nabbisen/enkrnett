//! Everything the daemon asks of the device, behind one trait. The OpenWrt implementation
//! (UCI, ubus, sysfs; chapter 6 and RFC 014) arrives with milestone M2. [`MockPlatform`] stands
//! in on a workstation and in CI and records what it was asked to do.

use std::net::IpAddr;

use enkrnett_model::api::{
    CheckResult, Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs,
    ManagementStatus, Memory, Radio, Storage, UplinkStatus,
};
use enkrnett_model::settings::Settings;
use enkrnett_model::state::OwnershipState;

mod mock;
pub use mock::MockPlatform;

/// A platform operation that did not succeed. The message is for the log, never for the
/// operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformError(pub String);

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "platform: {}", self.0)
    }
}

/// The effective state of the networks as the platform reports it (chapter 6 §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective {
    pub guest: GuestStatus,
    pub management: ManagementStatus,
    pub uplink: UplinkStatus,
}

/// Facts about the device for the details screen and the version line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub model: String,
    pub revision: String,
    pub tier: u8,
    pub device_class: String,
    pub memory: Memory,
    pub storage: Storage,
    pub hardening: Vec<HardeningItem>,
    pub installed_as: InstalledAs,
    pub radios: Vec<Radio>,
    pub baseline: String,
    pub openwrt: String,
}

pub trait Platform: Send + Sync {
    /// The last four characters of the address printed on the device's label (ENK-OWN-004,
    /// ENK-TIER-002).
    fn label_suffix(&self) -> String;

    /// Whether the device profile declares a light Enkrnett can control (ENK-OWN-006).
    fn has_light(&self) -> bool;

    /// Blinks the light for the identify step.
    fn blink(&self) -> Result<(), PlatformError>;

    /// The country when the device profile fixes it (ENK-RF-002).
    fn fixed_country(&self) -> Option<String>;

    /// The countries the operator may choose from when none is fixed.
    fn country_choices(&self) -> Vec<String>;

    /// The IPv4 address of the management UI, the same on the management and the setup network
    /// (ENK-MGMT-007).
    fn management_address(&self) -> String;

    /// How the requesting station is connected (RFC 016 §10).
    fn connection(&self, client: IpAddr, state: OwnershipState) -> Connection;

    /// Offer the setup network and nothing else.
    fn enter_setup(&self) -> Result<(), PlatformError>;

    /// Offer no network (the setup window closed, ENK-OWN-003).
    fn close_setup(&self) -> Result<(), PlatformError>;

    /// End the setup network and start the management network (RFC 004 step 8).
    fn enter_pending(
        &self,
        management_name: &str,
        management_key: &str,
    ) -> Result<(), PlatformError>;

    /// Bring the device to what the settings say, with the management network as given, and
    /// verify the result. An empty list means the health check passed.
    fn apply_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Vec<HealthDetail>, PlatformError>;

    /// What the device is doing now.
    fn effective(&self) -> Effective;

    fn device_info(&self) -> DeviceInfo;

    /// Fetches the release index (ENK-UPD-001) and reports the outcome.
    fn check_update_index(&self) -> CheckResult;

    /// Remove every platform setting Enkrnett owns (ENK-OWN-018).
    fn erase(&self) -> Result<(), PlatformError>;

    /// Reboot the device.
    fn restart(&self) -> Result<(), PlatformError>;
}

#[cfg(test)]
mod tests;
