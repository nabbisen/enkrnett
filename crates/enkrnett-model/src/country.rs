//! Country codes (chapter 5 §4; chapter 6 §2): two upper-case letters. Which codes a device
//! offers comes from its profile and the platform's regulatory database, not from this crate.

/// Whether `code` has the form of an ISO 3166-1 alpha-2 code.
pub fn is_valid_code(code: &str) -> bool {
    code.len() == 2 && code.bytes().all(|b| b.is_ascii_uppercase())
}
