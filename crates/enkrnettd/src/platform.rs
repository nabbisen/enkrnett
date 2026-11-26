//! Everything the daemon asks of the device, behind one trait. [`OpenWrtPlatform`] speaks UCI,
//! ubus, and sysfs (chapter 6 and RFC 014; `docs/src/design/platform.md`); [`MockPlatform`]
//! stands in on a workstation and in CI and records what it was asked to do.

use std::net::IpAddr;
use std::path::Path;

use enkrnett_model::api::{
    Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs, ManagementStatus, Memory,
    Radio, Storage, UpdateFailure, UplinkStatus,
};
use enkrnett_model::settings::Settings;
use enkrnett_model::state::OwnershipState;

mod mock;
pub mod openwrt;
pub mod spawn;
pub use mock::MockPlatform;
pub use openwrt::OpenWrtPlatform;

/// A platform operation that did not succeed. The message is for the log, never for the
/// operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformError {
    Failed(String),
    /// A configuration file the daemon owns is not writable by its user any more: a root-side
    /// `uci commit` handed it back to root. The init script restores ownership at every service
    /// start, so the daemon's answer is to exit and be respawned (`docs/src/design/platform.md`).
    NotOurs(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformError::Failed(m) => write!(f, "platform: {m}"),
            PlatformError::NotOurs(m) => write!(f, "platform: not ours: {m}"),
        }
    }
}

/// What taking over a device at start-up found (ENK-CFG-014: the daemon compares before it
/// writes, and does not overwrite drift by itself).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adopted {
    pub health: Vec<HealthDetail>,
    pub drift: bool,
}

/// The effective state of the networks as the platform reports it (chapter 6 §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective {
    pub guest: GuestStatus,
    pub management: ManagementStatus,
    pub uplink: UplinkStatus,
}

/// Why the platform's downloader could not deliver a file (ENK-UPD-001).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchError {
    /// No answer, an error status, a certificate that does not verify, a URL that is not
    /// `https://`: the message is for the log.
    Unreachable(String),
    /// More bytes than the caller allowed.
    TooLarge,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Unreachable(m) => write!(f, "unreachable: {m}"),
            FetchError::TooLarge => write!(f, "too large"),
        }
    }
}

/// What the root side of an update answered (chapter 11 §3). The daemon never runs as root: it
/// stages the verified image and asks; the platform's root helper verifies again, lets the
/// system check the image against the device, and flashes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallVerdict {
    /// The image is being written; the device restarts and this process ends.
    Flashing,
    /// Not written, with the reason for `last_result`.
    Refused(UpdateFailure),
    /// Writing started and did not succeed; the device still runs the old release.
    Failed,
}

/// Facts about the device for the details screen and the version line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    /// The device profile identifier, as the release index names it.
    pub profile: String,
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
    /// (ENK-MGMT-007); another one only after renumbering (ENK-TOPO-006).
    fn management_address(&self) -> String;

    /// Whether the daemon's listen address is derived from the management address at every
    /// start (the OpenWrt init script does that), so that a renumbered management network calls
    /// for a restart of the daemon. The mock listens wherever the tests say.
    fn listen_follows_management_address(&self) -> bool {
        false
    }

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

    /// Takes the device as the boot left it: if the owned sections already equal what the
    /// settings would produce, nothing is written and the result is verified; if they differ,
    /// that is drift, reported and left alone (ENK-CFG-014). A collision of the uplink's subnet
    /// with one of the networks is the one exception: the colliding network is renumbered and
    /// written (ENK-TOPO-003).
    fn adopt_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Adopted, PlatformError>;

    /// Whether the owned sections differ from what the daemon last wrote (ENK-CFG-014).
    fn owned_drift(&self) -> Result<bool, PlatformError>;

    /// Whether the uplink's subnet now collides with one of the networks (ENK-TOPO-003).
    fn uplink_collides(&self) -> bool;

    /// Disconnects every station of the management network with deauthentication
    /// (ENK-API-021), so that the next request proves a new association.
    fn disconnect_management_stations(&self) -> Result<(), PlatformError>;

    /// What the device is doing now.
    fn effective(&self) -> Effective;

    fn device_info(&self) -> DeviceInfo;

    /// Where the release index is (ENK-UPD-001); `None` when the platform configures none.
    fn update_index_url(&self) -> Option<String>;

    /// Fetches `url` through the platform's downloader over HTTPS, at most `max_bytes`.
    fn fetch(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError>;

    /// The signify public keys the image carries (ENK-RLS-005: the current key and the
    /// designated next one), as the texts of their files.
    fn release_keys(&self) -> Vec<String>;

    /// Downloads `url` through the platform's downloader over HTTPS into `dest` (in RAM), at
    /// most `max_bytes`: the image of chapter 11 §3. Returns the size written.
    fn download(&self, url: &str, dest: &Path, max_bytes: u64) -> Result<u64, FetchError>;

    /// Hands a verified image and its signature to the platform's root side (ENK-UPD-003 done
    /// once more where root can see it, then `sysupgrade` with the configuration kept,
    /// ENK-UPD-004). `token` names this attempt in the verdict. Returns when the request is
    /// placed, not when the image is written.
    fn install(&self, image: &Path, signature: &Path, token: &str) -> Result<(), PlatformError>;

    /// The root side's verdict on the attempt `token`, once there is one.
    fn install_verdict(&self, token: &str) -> Option<InstallVerdict>;

    /// Remove every platform setting Enkrnett owns (ENK-OWN-018).
    fn erase(&self) -> Result<(), PlatformError>;

    /// Reboot the device.
    fn restart(&self) -> Result<(), PlatformError>;

    /// Asks the service manager to restart the daemon's own service, so that the init script
    /// runs again (ownership of the owned files, the listen address). A plain exit would only
    /// be respawned with the same command line.
    fn restart_service(&self) -> Result<(), PlatformError>;
}

#[cfg(test)]
mod tests;
