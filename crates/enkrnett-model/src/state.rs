//! Ownership states (chapter 4 §1) and the connection enumerations of the greeting (chapter 9).

use serde::{Deserialize, Serialize};

/// The four ownership states. These are the only states (ENK-OWN-001 to 008).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OwnershipState {
    /// No owner. Setup window closed. No network is offered.
    Unclaimed,
    /// No owner. Setup window open. The setup network is offered.
    Setup,
    /// Setup was confirmed. The claim is not final yet. The management network is offered.
    Pending,
    /// The device has an owner.
    Claimed,
}

impl OwnershipState {
    /// The value as the API writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            OwnershipState::Unclaimed => "UNCLAIMED",
            OwnershipState::Setup => "SETUP",
            OwnershipState::Pending => "PENDING",
            OwnershipState::Claimed => "CLAIMED",
        }
    }
}

/// Which of the device's networks a request arrived on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionNetwork {
    Setup,
    Management,
}

/// How the requesting station's radio link is protected (RFC 016 §10). `None` means an open,
/// unencrypted association. The daemon sends `null` when it cannot determine the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionSecurity {
    None,
    Owe,
    Wpa2,
    Wpa3,
}
