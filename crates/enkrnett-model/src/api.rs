//! The shapes of the management API (chapter 9 §4) and the error shape (§5). Every shape is
//! defined once here and embedded whole where it appears inside another (ENK-API-009). Field
//! names are `snake_case`; every field is always present; `null` only where the specification
//! says so (ENK-API-004).

use serde::{Deserialize, Serialize};

use crate::settings::{GuestMode, Limits, Settings};
use crate::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};

// ----- greeting, connection, session --------------------------------------------------------

/// `connection` (RFC 016 §10).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub network: ConnectionNetwork,
    pub security: Option<ConnectionSecurity>,
}

/// The greeting, the only resource readable without a session (ENK-API-011).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Greeting {
    pub product: &'static str,
    pub api: u32,
    pub state: OwnershipState,
    pub languages: Vec<String>,
    pub development: bool,
    pub connection: Connection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    Claim,
    Owner,
}

/// `session`: returned by `claim/start` and `login`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub kind: SessionKind,
    pub state: OwnershipState,
    pub idle_ends_in: u64,
    pub ends_in: u64,
}

// ----- claim ----------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryChoice {
    pub fixed: Option<String>,
    pub choices: Vec<String>,
}

/// `claim`: what setup needs to know.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub window_ends_in: u64,
    pub country: CountryChoice,
    pub guest_mode_default: GuestMode,
    pub identify_available: bool,
    pub limits: Limits,
    pub connection: Connection,
}

/// `claim/check` request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimCheck {
    #[serde(default)]
    pub country: Option<String>,
    pub guest: ClaimGuest,
    pub owner_password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimGuest {
    pub name: String,
    pub mode: GuestMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementProposal {
    pub name: String,
    pub key: String,
}

/// `proposal`: returned by `claim/check` and `claim/confirm`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub management: ManagementProposal,
    pub address: String,
    pub guest: ClaimGuest,
    pub country: String,
    pub state: OwnershipState,
    pub final_in: Option<u64>,
}

// ----- settings, change -----------------------------------------------------------------------

/// `settings` as the API returns it: the document plus `limits`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsShape {
    #[serde(flatten)]
    pub settings: Settings,
    pub limits: Limits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeState {
    None,
    Validating,
    Applying,
    WaitingConfirmation,
    Settled,
    Undone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Settings,
    Restore,
    ManagementKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UndoReason {
    HealthCheckFailed,
    NotConfirmed,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthDetail {
    GuestDown,
    ManagementDown,
    IsolationMissing,
    DhcpMissing,
    ListenerExposed,
    PmfMissing,
    ModeDiffers,
}

/// `change`: the current or the last transaction. When `state` is `none`, every field but `id`
/// and `state` is `null`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub id: u64,
    pub state: ChangeState,
    pub kind: Option<ChangeKind>,
    pub from_version: Option<u64>,
    pub to_version: Option<u64>,
    pub needs_confirmation: Option<bool>,
    pub confirm_in: Option<u64>,
    pub reason: Option<UndoReason>,
    pub detail: Option<HealthDetail>,
    pub started_ago: Option<u64>,
}

impl Change {
    /// The `change` shape when no transaction exists or ever existed.
    pub fn none() -> Change {
        Change {
            id: 0,
            state: ChangeState::None,
            kind: None,
            from_version: None,
            to_version: None,
            needs_confirmation: None,
            confirm_in: None,
            reason: None,
            detail: None,
            started_ago: None,
        }
    }
}

// ----- status, device, update -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub enkrnett: String,
    pub baseline: String,
    pub openwrt: String,
    pub api: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestStatus {
    pub up: bool,
    pub name: Option<String>,
    pub mode: Option<GuestMode>,
    pub clients_encrypted: u64,
    pub clients_unencrypted: u64,
    pub checked_ago: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementStatus {
    pub up: bool,
    pub name: Option<String>,
    pub address: String,
    pub clients: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UplinkStatus {
    pub up: bool,
    pub address: Option<String>,
}

/// `status`: the first screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub state: OwnershipState,
    pub version: Version,
    pub settings: SettingsShape,
    pub guest: GuestStatus,
    pub management: ManagementStatus,
    pub uplink: UplinkStatus,
    pub change: Change,
    pub drift: bool,
    pub update: Update,
    pub hardening_met: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub total_kib: u64,
    pub available_kib: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Storage {
    pub overlay_total_kib: u64,
    pub overlay_free_kib: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardeningItem {
    pub id: String,
    pub met: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstalledAs {
    Image,
    Package,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Radio {
    pub band: String,
    pub in_use: bool,
}

/// `device`: the details screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub model: String,
    pub revision: String,
    pub tier: u8,
    pub device_class: String,
    pub device_id: String,
    pub version: Version,
    pub uptime_seconds: u64,
    pub memory: Memory,
    pub storage: Storage,
    pub hardening: Vec<HardeningItem>,
    pub installed_as: InstalledAs,
    pub radios: Vec<Radio>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Ok,
    Unreachable,
    BadSignature,
    BadIndex,
    StaleIndex,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastCheck {
    pub checked_ago: u64,
    pub result: CheckResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    pub version: String,
    pub baseline: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Available {
    pub version: String,
    pub baseline: String,
    pub changes_baseline: bool,
    pub size_kib: u64,
    pub notes_url: String,
    pub install_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    Idle,
    Downloading,
    Verifying,
    Installing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateOutcome {
    Done,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateFailure {
    BadSignature,
    WrongDevice,
    WrongBaseline,
    NotEnoughMemory,
    DownloadFailed,
    HealthCheckFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastResult {
    pub result: UpdateOutcome,
    pub reason: Option<UpdateFailure>,
}

/// `update`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub installed: Installed,
    pub checks_enabled: bool,
    pub last_check: Option<LastCheck>,
    pub available: Option<Available>,
    pub state: UpdateState,
    pub last_result: Option<LastResult>,
    pub can_undo: bool,
}

/// `management_key`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementKey {
    pub name: String,
    pub key: String,
}

// ----- requests -------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordChange {
    pub current: String,
    pub new: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EraseRequest {
    pub password: String,
}

/// The body of every `POST` that has no parameters (ENK-API-002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

// ----- errors ---------------------------------------------------------------------------------

/// The code of an error response (chapter 9 §5). A new code requires an RFC (ENK-API-018).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Invalid,
    CannotUpdate,
    NoSession,
    WrongPassword,
    WrongState,
    NotAvailable,
    UnknownResource,
    MethodNotAllowed,
    SetupInProgressElsewhere,
    NoProposal,
    VersionMismatch,
    ChangePending,
    NothingToInstall,
    UpdateInProgress,
    TooLarge,
    WrongContentType,
    TryLater,
    Internal,
}

impl ErrorCode {
    /// The HTTP status that carries the code.
    pub fn http_status(self) -> u16 {
        match self {
            ErrorCode::Invalid | ErrorCode::CannotUpdate => 400,
            ErrorCode::NoSession | ErrorCode::WrongPassword => 401,
            ErrorCode::WrongState | ErrorCode::NotAvailable => 403,
            ErrorCode::UnknownResource => 404,
            ErrorCode::MethodNotAllowed => 405,
            ErrorCode::SetupInProgressElsewhere
            | ErrorCode::NoProposal
            | ErrorCode::VersionMismatch
            | ErrorCode::ChangePending
            | ErrorCode::NothingToInstall
            | ErrorCode::UpdateInProgress => 409,
            ErrorCode::TooLarge => 413,
            ErrorCode::WrongContentType => 415,
            ErrorCode::TryLater => 429,
            ErrorCode::Internal => 500,
        }
    }
}

/// The code of one failing field inside an `invalid` error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldCode {
    Required,
    UnknownField,
    WrongType,
    TooShort,
    TooLong,
    NotAllowed,
    ForbiddenCharacters,
    CommonPassword,
    EqualsNetworkName,
    EqualsKey,
    EqualsDeviceId,
    NotInChoices,
}

/// One failing field (ENK-API-019).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub code: FieldCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

impl FieldError {
    pub fn new(field: &str, code: FieldCode) -> FieldError {
        FieldError {
            field: field.to_string(),
            code,
            limit: None,
        }
    }

    pub fn with_limit(field: &str, code: FieldCode, limit: u64) -> FieldError {
        FieldError {
            field: field.to_string(),
            code,
            limit: Some(limit),
        }
    }
}

/// The one error shape (ENK-API-018). `message` is English for developers and never shown to
/// the operator (ENK-API-007). Extra fields that a code carries (`state`, `retry_in`,
/// `reason`, `reference`, `settings`, `change`) are serialised beside `code`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<FieldError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<OwnershipState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<UpdateFailure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<Change>,
}

impl ErrorDetail {
    pub fn new(code: ErrorCode) -> ErrorDetail {
        ErrorDetail {
            code,
            fields: Vec::new(),
            message: None,
            state: None,
            retry_in: None,
            reason: None,
            reference: None,
            settings: None,
            change: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> ErrorDetail {
        self.message = Some(message.into());
        self
    }
}

/// `{ "error": { ... } }`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[cfg(test)]
mod tests;
