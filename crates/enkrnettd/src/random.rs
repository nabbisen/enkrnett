//! Random values from the operating system: session tokens (ENK-AUTH-002), management keys
//! (ENK-OWN-014), device identifiers (ENK-OWN-020).

use enkrnett_model::device::{ID_BYTES, device_id_from_random};
use enkrnett_model::encoding::base64url;
use enkrnett_model::key::management_key_from_random;
use rand_core::{OsRng, RngCore};

/// `N` random bytes.
pub fn bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    OsRng.fill_bytes(&mut b);
    b
}

/// A session token: 32 random bytes as base64url, 43 characters.
pub fn session_token() -> String {
    base64url(&bytes::<32>())
}

/// A management key in the form of `enkrnett_model::key`.
pub fn management_key() -> String {
    loop {
        if let Some(key) = management_key_from_random(bytes::<64>().into_iter()) {
            return key;
        }
    }
}

/// A device identifier.
pub fn device_id() -> String {
    device_id_from_random(&bytes::<ID_BYTES>())
}
