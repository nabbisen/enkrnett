//! ENK-OWN-015.

use super::*;

#[test]
fn the_alphabet_has_no_confusable_symbols() {
    for forbidden in b"0o1liOIL" {
        assert!(
            !ALPHABET.contains(forbidden),
            "{} is confusable",
            *forbidden as char
        );
    }
    let mut sorted = ALPHABET.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 31);
}

#[test]
fn form_and_length() {
    let key = management_key_from_random((0u8..=255).cycle()).expect("enough bytes");
    assert_eq!(key.len(), 23);
    assert!(key.len() <= 24);
    assert!(is_management_key(&key));
    assert_eq!(key.matches('-').count(), 3);
}

#[test]
fn at_least_eighty_bits() {
    assert!(entropy_bits() >= 80.0);
    assert!((entropy_bits() - 99.08).abs() < 0.1);
}

#[test]
fn rejection_sampling_skips_biased_bytes() {
    // Bytes 248..=255 are rejected; only the final 20 accepted bytes build the key.
    let stream = [250u8, 255, 0, 1, 2]
        .into_iter()
        .chain(std::iter::repeat(3))
        .take(40);
    let key = management_key_from_random(stream).unwrap();
    assert!(key.starts_with("abc"));
}

#[test]
fn too_few_bytes_gives_none() {
    assert!(management_key_from_random([1u8, 2, 3].into_iter()).is_none());
}

#[test]
fn recognises_only_its_own_form() {
    assert!(is_management_key("abcde-fghjk-mnpqr-stuvw"));
    assert!(!is_management_key("abcde-fghjk-mnpqr-stuv"));
    assert!(!is_management_key("abcde-fghjk-mnpqr-stuv0"));
    assert!(!is_management_key("abcdefghjkmnpqrstuvw"));
}
