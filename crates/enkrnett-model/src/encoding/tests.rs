use super::*;

#[test]
fn base64url_matches_rfc_4648_vectors() {
    assert_eq!(base64url(b""), "");
    assert_eq!(base64url(b"f"), "Zg");
    assert_eq!(base64url(b"fo"), "Zm8");
    assert_eq!(base64url(b"foo"), "Zm9v");
    assert_eq!(base64url(b"foob"), "Zm9vYg");
    assert_eq!(base64url(b"fooba"), "Zm9vYmE");
    assert_eq!(base64url(b"foobar"), "Zm9vYmFy");
    assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
}

#[test]
fn thirty_two_bytes_make_forty_three_characters() {
    assert_eq!(base64url(&[0u8; 32]).len(), 43);
}

#[test]
fn hex_is_lower_case_and_fixed_width() {
    assert_eq!(hex(&[0x00, 0x0f, 0xa0, 0xff]), "000fa0ff");
    assert_eq!(hex(&[]), "");
}
