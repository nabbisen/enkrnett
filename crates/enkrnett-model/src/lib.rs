//! The Enkrnett model: the settings document, its validation, the API shapes, and the pure
//! functions behind generated values. This crate does no I/O and knows nothing about HTTP,
//! OpenWrt, or time. Everything here is derived from the consolidated specification in
//! `docs/src/specification/` (chapters 1, 4, 6, 7, 9).
//!
//! The daemon (`enkrnettd`) and the test tools depend on it; the UI's JSON Schema is generated
//! from the shapes in [`api`].

pub mod api;
pub mod country;
pub mod device;
pub mod encoding;
pub mod key;
pub mod password;
pub mod settings;
pub mod state;

/// The API version carried in the path prefix `/api/1/` and in the greeting (ENK-API-001).
pub const API_VERSION: u32 = 1;

/// The product name carried in the greeting.
pub const PRODUCT: &str = "Enkrnett";

/// Format version of everything the daemon stores (ENK-BASE-005). Raised when a stored shape
/// changes incompatibly; a newer daemon must read every earlier version.
pub const STORAGE_FORMAT: u32 = 1;
