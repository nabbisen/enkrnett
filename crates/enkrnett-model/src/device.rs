//! The device identifier (ENK-OWN-020): random, created when the device has none, not a
//! secret, erased by Erase.

use crate::encoding::hex;

/// Bytes of randomness in an identifier.
pub const ID_BYTES: usize = 8;

/// Builds the identifier from random bytes: 16 lower-case hexadecimal characters.
pub fn device_id_from_random(random: &[u8; ID_BYTES]) -> String {
    hex(random)
}

/// Whether a string has the form of an identifier.
pub fn is_device_id(candidate: &str) -> bool {
    candidate.len() == ID_BYTES * 2
        && candidate
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
