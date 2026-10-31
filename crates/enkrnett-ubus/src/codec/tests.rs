use serde_json::json;

use super::*;

#[test]
fn attr_encoding_pads_to_four_bytes() {
    let a = attr_encode(attr::OBJPATH, b"ab");
    assert_eq!(a, vec![0x02, 0x00, 0x00, 0x06, b'a', b'b', 0, 0]);
    let decoded = attrs_decode(&a).unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].id, attr::OBJPATH);
    assert_eq!(decoded[0].payload, b"ab");
    assert!(!decoded[0].extended);
}

#[test]
fn message_round_trip() {
    let path = attr_encode(attr::OBJPATH, &cstring("system"));
    let bytes = Message::encode(MessageType::Lookup, 7, 0, std::slice::from_ref(&path));
    assert_eq!(&bytes[..8], &[0, 4, 0, 7, 0, 0, 0, 0]);
    let (msg, used) = Message::decode(&bytes).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(msg.kind, MessageType::Lookup);
    assert_eq!(msg.seq, 7);
    assert_eq!(msg.body, path);
    let mut prefix = [0u8; 12];
    prefix.copy_from_slice(&bytes[..12]);
    assert_eq!(Message::size_from_prefix(&prefix).unwrap(), bytes.len());
}

#[test]
fn malformed_attributes_are_errors_not_panics() {
    assert!(attrs_decode(&[0, 0, 0]).is_err());
    assert!(attrs_decode(&[0, 0, 0, 2]).is_err());
    assert!(attrs_decode(&[0, 0, 0, 9, 1, 2, 3, 4]).is_err());
    assert!(Message::decode(&[0u8; 11]).is_err());
    assert!(Message::decode(&[0, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]).is_err());
}

#[test]
fn blobmsg_string_known_vector() {
    // name "ssid" (4 bytes), data "x\0": header u16 len=4, "ssid", NUL, pad to 8; data "x\0" pad.
    let body = blobmsg_encode_table(&json!({"ssid": "x"}).as_object().unwrap().clone());
    let word = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
    assert_ne!(word & 0x8000_0000, 0, "extended bit");
    assert_eq!((word & 0x7f00_0000) >> 24, blobmsg_type::STRING);
    assert_eq!(&body[4..6], &[0, 4]);
    assert_eq!(&body[6..10], b"ssid");
    assert_eq!(body[10], 0);
    assert_eq!(&body[12..14], b"x\0");
    let back = blobmsg_decode_table(&body).unwrap();
    assert_eq!(back.get("ssid"), Some(&Value::String("x".into())));
}

#[test]
fn blobmsg_round_trip_of_nested_values() {
    let value = json!({
        "up": true,
        "pending": false,
        "count": 12,
        "big": 1_i64 << 40,
        "negative": -5,
        "name": "Cafe Example",
        "list": [1, "two", {"three": 3}],
        "nested": {"a": {"b": null}}
    });
    let body = blobmsg_encode_table(value.as_object().unwrap());
    let back = Value::Object(blobmsg_decode_table(&body).unwrap());
    assert_eq!(back, value);
}

#[test]
fn blobmsg_rejects_truncated_names() {
    let mut body = blobmsg_encode_table(&json!({"ssid": "x"}).as_object().unwrap().clone());
    body[5] = 200; // name length beyond the payload
    assert!(blobmsg_decode_table(&body).is_err());
}

#[test]
fn u32_payload_reads_big_endian() {
    assert_eq!(u32_payload(&[0, 0, 1, 2]), Some(258));
    assert_eq!(u32_payload(&[1, 2]), None);
}
