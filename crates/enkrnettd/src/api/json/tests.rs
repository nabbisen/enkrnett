//! ENK-SEC-020.

use super::*;

#[test]
fn accepts_ordinary_documents() {
    assert_eq!(check_strict(br#"{}"#), Ok(()));
    assert_eq!(
        check_strict(br#"{"password":"x","n":1,"b":true,"z":null,"list":[1,2,{"a":"b"}]}"#),
        Ok(())
    );
    assert_eq!(
        check_strict(br#"{"big":9223372036854775807,"neg":-1}"#),
        Ok(())
    );
}

#[test]
fn refuses_duplicate_keys_anywhere() {
    assert_eq!(check_strict(br#"{"a":1,"a":2}"#), Err("duplicate key"));
    assert_eq!(
        check_strict(br#"{"x":{"a":1,"a":2}}"#),
        Err("duplicate key")
    );
}

#[test]
fn refuses_fractions_exponents_and_huge_numbers() {
    assert_eq!(
        check_strict(br#"{"a":1.5}"#),
        Err("numbers must be integers within 64 bits")
    );
    assert_eq!(
        check_strict(br#"{"a":1e3}"#),
        Err("numbers must be integers within 64 bits")
    );
    assert_eq!(
        check_strict(br#"{"a":99999999999999999999999}"#),
        Err("numbers must be integers within 64 bits")
    );
}

#[test]
fn refuses_deep_nesting() {
    let ok = format!("{}1{}", "[".repeat(8), "]".repeat(8));
    assert_eq!(check_strict(ok.as_bytes()), Ok(()));
    let deep = format!("{}1{}", "[".repeat(9), "]".repeat(9));
    assert_eq!(check_strict(deep.as_bytes()), Err("nesting deeper than 8"));
}

#[test]
fn refuses_garbage_and_trailing_text() {
    assert_eq!(check_strict(b"not json"), Err("body is not valid JSON"));
    assert_eq!(check_strict(b""), Err("body is not valid JSON"));
    assert_eq!(
        check_strict(b"{} {}"),
        Err("trailing characters after the JSON value")
    );
}
