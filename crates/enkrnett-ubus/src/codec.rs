//! The libubus wire format.
//!
//! A message is an 8-byte header `{version u8, type u8, seq u16 BE, peer u32 BE}` followed by one
//! top-level `blob_attr` whose payload is a list of `blob_attr`s. A `blob_attr` is a 4-byte
//! big-endian word `{extended bit 31, id bits 24..30, len bits 0..23 (including the word)}`
//! followed by the payload, padded to 4 bytes. Method arguments and replies (`ATTR_DATA`) are
//! `blobmsg`: `blob_attr`s with the extended bit set, whose payload starts with a name
//! (`u16 BE length`, the name, a NUL, padding to 4) followed by the typed data.

use serde_json::{Map, Number, Value};

/// Message types of the ubus protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Hello = 0,
    Status = 1,
    Data = 2,
    Ping = 3,
    Lookup = 4,
    Invoke = 5,
    AddObject = 6,
    RemoveObject = 7,
    Subscribe = 8,
    Unsubscribe = 9,
    Notify = 10,
    Monitor = 11,
}

impl MessageType {
    pub fn from_u8(v: u8) -> Option<MessageType> {
        use MessageType::*;
        Some(match v {
            0 => Hello,
            1 => Status,
            2 => Data,
            3 => Ping,
            4 => Lookup,
            5 => Invoke,
            6 => AddObject,
            7 => RemoveObject,
            8 => Subscribe,
            9 => Unsubscribe,
            10 => Notify,
            11 => Monitor,
            _ => return None,
        })
    }
}

/// Attribute identifiers of ubus messages.
pub mod attr {
    pub const UNSPEC: u32 = 0;
    pub const STATUS: u32 = 1;
    pub const OBJPATH: u32 = 2;
    pub const OBJID: u32 = 3;
    pub const METHOD: u32 = 4;
    pub const OBJTYPE: u32 = 5;
    pub const SIGNATURE: u32 = 6;
    pub const DATA: u32 = 7;
    pub const TARGET: u32 = 8;
    pub const ACTIVE: u32 = 9;
    pub const NO_REPLY: u32 = 10;
}

/// blobmsg data types.
pub mod blobmsg_type {
    pub const UNSPEC: u32 = 0;
    pub const ARRAY: u32 = 1;
    pub const TABLE: u32 = 2;
    pub const STRING: u32 = 3;
    pub const INT64: u32 = 4;
    pub const INT32: u32 = 5;
    pub const INT16: u32 = 6;
    pub const INT8: u32 = 7;
    pub const DOUBLE: u32 = 8;
}

const EXTENDED: u32 = 0x8000_0000;
const ID_MASK: u32 = 0x7f00_0000;
const LEN_MASK: u32 = 0x00ff_ffff;

/// A decoding failure: the bytes did not have the described form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError(pub &'static str);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ubus decode: {}", self.0)
    }
}

impl std::error::Error for DecodeError {}

fn pad4(len: usize) -> usize {
    (len + 3) & !3
}

/// Encodes one plain `blob_attr`: id, payload, padded to 4 bytes.
pub fn attr_encode(id: u32, payload: &[u8]) -> Vec<u8> {
    let len = 4 + payload.len();
    let mut v = (((id << 24) & ID_MASK) | len as u32).to_be_bytes().to_vec();
    v.extend_from_slice(payload);
    v.resize(pad4(v.len()), 0);
    v
}

/// One decoded `blob_attr`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr<'a> {
    pub id: u32,
    pub extended: bool,
    pub payload: &'a [u8],
}

/// Decodes a list of `blob_attr`s from a payload.
pub fn attrs_decode(payload: &[u8]) -> Result<Vec<Attr<'_>>, DecodeError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < payload.len() {
        if i + 4 > payload.len() {
            return Err(DecodeError("truncated attribute header"));
        }
        let word = u32::from_be_bytes([payload[i], payload[i + 1], payload[i + 2], payload[i + 3]]);
        let len = (word & LEN_MASK) as usize;
        if len < 4 || i + len > payload.len() {
            return Err(DecodeError("attribute length out of range"));
        }
        out.push(Attr {
            id: (word & ID_MASK) >> 24,
            extended: word & EXTENDED != 0,
            payload: &payload[i + 4..i + len],
        });
        i += pad4(len);
    }
    Ok(out)
}

/// A whole message: header fields and the attributes of its top-level blob.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub kind: MessageType,
    pub seq: u16,
    pub peer: u32,
    /// The payload of the top-level blob (a list of attributes).
    pub body: Vec<u8>,
}

impl Message {
    /// Encodes a message whose body is the concatenation of `attrs` (already encoded).
    pub fn encode(kind: MessageType, seq: u16, peer: u32, attrs: &[Vec<u8>]) -> Vec<u8> {
        let body: Vec<u8> = attrs.concat();
        let mut out = vec![0u8, kind as u8];
        out.extend_from_slice(&seq.to_be_bytes());
        out.extend_from_slice(&peer.to_be_bytes());
        out.extend_from_slice(&attr_encode(attr::UNSPEC, &body));
        out
    }

    /// Decodes a message from bytes that hold exactly one message. Returns the message and the
    /// number of bytes it occupied.
    pub fn decode(bytes: &[u8]) -> Result<(Message, usize), DecodeError> {
        if bytes.len() < 12 {
            return Err(DecodeError("short message"));
        }
        let kind = MessageType::from_u8(bytes[1]).ok_or(DecodeError("unknown message type"))?;
        let seq = u16::from_be_bytes([bytes[2], bytes[3]]);
        let peer = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let word = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let len = (word & LEN_MASK) as usize;
        if len < 4 || 8 + len > bytes.len() {
            return Err(DecodeError("top-level blob length out of range"));
        }
        let body = bytes[12..8 + len].to_vec();
        Ok((
            Message {
                kind,
                seq,
                peer,
                body,
            },
            8 + pad4(len),
        ))
    }

    /// Size of the whole message given its first 12 bytes, so that a reader knows how much to
    /// read.
    pub fn size_from_prefix(prefix: &[u8; 12]) -> Result<usize, DecodeError> {
        let word = u32::from_be_bytes([prefix[8], prefix[9], prefix[10], prefix[11]]);
        let len = (word & LEN_MASK) as usize;
        if len < 4 {
            return Err(DecodeError("top-level blob length out of range"));
        }
        Ok(8 + pad4(len))
    }
}

// ----- blobmsg <-> JSON -----------------------------------------------------------------------

fn name_header(name: &str) -> Vec<u8> {
    let mut v = (name.len() as u16).to_be_bytes().to_vec();
    v.extend_from_slice(name.as_bytes());
    v.push(0);
    v.resize(pad4(v.len()), 0);
    v
}

fn blobmsg_attr(kind: u32, name: &str, data: &[u8]) -> Vec<u8> {
    let mut payload = name_header(name);
    payload.extend_from_slice(data);
    let len = 4 + payload.len();
    let mut v = (EXTENDED | ((kind << 24) & ID_MASK) | len as u32)
        .to_be_bytes()
        .to_vec();
    v.extend_from_slice(&payload);
    v.resize(pad4(v.len()), 0);
    v
}

fn blobmsg_value(name: &str, value: &Value) -> Vec<u8> {
    match value {
        Value::Null => blobmsg_attr(blobmsg_type::UNSPEC, name, &[]),
        Value::Bool(b) => blobmsg_attr(blobmsg_type::INT8, name, &[u8::from(*b)]),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i32::try_from(i).is_ok() {
                    blobmsg_attr(blobmsg_type::INT32, name, &(i as i32).to_be_bytes())
                } else {
                    blobmsg_attr(blobmsg_type::INT64, name, &i.to_be_bytes())
                }
            } else if let Some(u) = n.as_u64() {
                blobmsg_attr(blobmsg_type::INT64, name, &(u as i64).to_be_bytes())
            } else {
                blobmsg_attr(
                    blobmsg_type::DOUBLE,
                    name,
                    &n.as_f64().unwrap_or(0.0).to_be_bytes(),
                )
            }
        }
        Value::String(s) => {
            let mut d = s.as_bytes().to_vec();
            d.push(0);
            blobmsg_attr(blobmsg_type::STRING, name, &d)
        }
        Value::Array(items) => {
            let inner: Vec<u8> = items.iter().flat_map(|v| blobmsg_value("", v)).collect();
            blobmsg_attr(blobmsg_type::ARRAY, name, &inner)
        }
        Value::Object(map) => blobmsg_attr(blobmsg_type::TABLE, name, &blobmsg_table_body(map)),
    }
}

fn blobmsg_table_body(map: &Map<String, Value>) -> Vec<u8> {
    map.iter().flat_map(|(k, v)| blobmsg_value(k, v)).collect()
}

/// Encodes a JSON object as the body of a blobmsg table (a list of named attributes), as a ubus
/// method's arguments are sent inside `ATTR_DATA`.
pub fn blobmsg_encode_table(map: &Map<String, Value>) -> Vec<u8> {
    blobmsg_table_body(map)
}

fn read_name(payload: &[u8]) -> Result<(&str, &[u8]), DecodeError> {
    if payload.len() < 2 {
        return Err(DecodeError("blobmsg name header missing"));
    }
    let name_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let end = 2 + name_len;
    if end + 1 > payload.len() {
        return Err(DecodeError("blobmsg name out of range"));
    }
    let name =
        std::str::from_utf8(&payload[2..end]).map_err(|_| DecodeError("blobmsg name not UTF-8"))?;
    let data_start = pad4(end + 1);
    if data_start > payload.len() {
        return Err(DecodeError("blobmsg data missing"));
    }
    Ok((name, &payload[data_start..]))
}

fn blobmsg_data_to_json(kind: u32, data: &[u8]) -> Result<Value, DecodeError> {
    Ok(match kind {
        blobmsg_type::UNSPEC => Value::Null,
        blobmsg_type::INT8 => match data.first() {
            Some(0) => Value::Bool(false),
            Some(_) => Value::Bool(true),
            None => return Err(DecodeError("int8 without data")),
        },
        blobmsg_type::INT16 => {
            let b: [u8; 2] = data
                .get(..2)
                .and_then(|d| d.try_into().ok())
                .ok_or(DecodeError("int16 short"))?;
            Value::from(i16::from_be_bytes(b))
        }
        blobmsg_type::INT32 => {
            let b: [u8; 4] = data
                .get(..4)
                .and_then(|d| d.try_into().ok())
                .ok_or(DecodeError("int32 short"))?;
            Value::from(i32::from_be_bytes(b))
        }
        blobmsg_type::INT64 => {
            let b: [u8; 8] = data
                .get(..8)
                .and_then(|d| d.try_into().ok())
                .ok_or(DecodeError("int64 short"))?;
            Value::from(i64::from_be_bytes(b))
        }
        blobmsg_type::DOUBLE => {
            let b: [u8; 8] = data
                .get(..8)
                .and_then(|d| d.try_into().ok())
                .ok_or(DecodeError("double short"))?;
            Number::from_f64(f64::from_be_bytes(b))
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        blobmsg_type::STRING => {
            let s = data.split(|b| *b == 0).next().unwrap_or(&[]);
            Value::String(String::from_utf8_lossy(s).into_owned())
        }
        blobmsg_type::ARRAY => {
            let mut items = Vec::new();
            for a in attrs_decode(data)? {
                let (_, d) = if a.extended {
                    read_name(a.payload)?
                } else {
                    ("", a.payload)
                };
                items.push(blobmsg_data_to_json(a.id, d)?);
            }
            Value::Array(items)
        }
        blobmsg_type::TABLE => Value::Object(blobmsg_decode_table(data)?),
        _ => return Err(DecodeError("unknown blobmsg type")),
    })
}

/// Decodes the body of a blobmsg table into a JSON object.
pub fn blobmsg_decode_table(body: &[u8]) -> Result<Map<String, Value>, DecodeError> {
    let mut map = Map::new();
    for a in attrs_decode(body)? {
        let (name, data) = if a.extended {
            read_name(a.payload)?
        } else {
            ("", a.payload)
        };
        map.insert(name.to_string(), blobmsg_data_to_json(a.id, data)?);
    }
    Ok(map)
}

/// A NUL-terminated string as `ATTR_OBJPATH` and `ATTR_METHOD` carry it.
pub fn cstring(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}

/// Reads a `u32` attribute payload.
pub fn u32_payload(p: &[u8]) -> Option<u32> {
    p.get(..4)
        .and_then(|d| d.try_into().ok())
        .map(u32::from_be_bytes)
}

#[cfg(test)]
mod tests;
