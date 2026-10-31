//! Talks to `ubusd` on its unix socket: HELLO, LOOKUP, INVOKE. Blocking, one request at a time,
//! one connection per client. The daemon opens a client per call site and drops it; `ubusd`
//! handles that cheaply and it keeps the client free of state.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use serde_json::{Map, Value};

use crate::codec::{self, Message, MessageType, attr, cstring, u32_payload};

/// Where `ubusd` listens on OpenWrt, in order of preference.
pub const SOCKETS: [&str; 2] = ["/var/run/ubus/ubus.sock", "/var/run/ubus.sock"];

/// A failure to talk to `ubusd` or a non-zero status from the object.
#[derive(Debug)]
pub enum UbusError {
    Io(std::io::Error),
    Decode(codec::DecodeError),
    /// `ubusd` answered with a status other than 0 (`UBUS_STATUS_*`).
    Status(u32),
    Protocol(&'static str),
    NotFound(String),
}

impl std::fmt::Display for UbusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UbusError::Io(e) => write!(f, "ubus io: {e}"),
            UbusError::Decode(e) => write!(f, "{e}"),
            UbusError::Status(s) => write!(f, "ubus status {s}"),
            UbusError::Protocol(s) => write!(f, "ubus protocol: {s}"),
            UbusError::NotFound(p) => write!(f, "ubus object not found: {p}"),
        }
    }
}

impl std::error::Error for UbusError {}

impl From<std::io::Error> for UbusError {
    fn from(e: std::io::Error) -> Self {
        UbusError::Io(e)
    }
}

impl From<codec::DecodeError> for UbusError {
    fn from(e: codec::DecodeError) -> Self {
        UbusError::Decode(e)
    }
}

/// One object as LOOKUP reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectInfo {
    pub path: String,
    pub id: u32,
    pub type_id: u32,
}

/// A connected client.
pub struct Client {
    stream: UnixStream,
    seq: u16,
}

impl Client {
    /// Connects to the first socket of [`SOCKETS`] that answers and reads HELLO.
    pub fn connect() -> Result<Client, UbusError> {
        let mut last = None;
        for path in SOCKETS {
            match UnixStream::connect(path) {
                Ok(s) => return Client::from_stream(s),
                Err(e) => last = Some(e),
            }
        }
        Err(UbusError::Io(
            last.unwrap_or_else(|| std::io::Error::other("no socket")),
        ))
    }

    /// Connects to a given socket path (tests and non-standard layouts).
    pub fn connect_path(path: &str) -> Result<Client, UbusError> {
        Client::from_stream(UnixStream::connect(path)?)
    }

    fn from_stream(stream: UnixStream) -> Result<Client, UbusError> {
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        let mut client = Client { stream, seq: 0 };
        let hello = client.read_message()?;
        if hello.kind != MessageType::Hello {
            return Err(UbusError::Protocol("expected HELLO"));
        }
        Ok(client)
    }

    fn next_seq(&mut self) -> u16 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    fn send(&mut self, kind: MessageType, peer: u32, attrs: &[Vec<u8>]) -> Result<u16, UbusError> {
        let seq = self.next_seq();
        self.stream
            .write_all(&Message::encode(kind, seq, peer, attrs))?;
        Ok(seq)
    }

    fn read_message(&mut self) -> Result<Message, UbusError> {
        let mut prefix = [0u8; 12];
        self.stream.read_exact(&mut prefix)?;
        let size = Message::size_from_prefix(&prefix)?;
        let mut bytes = prefix.to_vec();
        bytes.resize(size, 0);
        self.stream.read_exact(&mut bytes[12..])?;
        Ok(Message::decode(&bytes)?.0)
    }

    /// Lists the objects whose path equals `path` (or every object when `path` is empty).
    pub fn lookup(&mut self, path: &str) -> Result<Vec<ObjectInfo>, UbusError> {
        let attrs = if path.is_empty() {
            vec![]
        } else {
            vec![codec::attr_encode(attr::OBJPATH, &cstring(path))]
        };
        let seq = self.send(MessageType::Lookup, 0, &attrs)?;
        let mut found = Vec::new();
        loop {
            let msg = self.read_message()?;
            if msg.seq != seq {
                continue;
            }
            match msg.kind {
                MessageType::Data => {
                    let mut info = ObjectInfo {
                        path: String::new(),
                        id: 0,
                        type_id: 0,
                    };
                    for a in codec::attrs_decode(&msg.body)? {
                        match a.id {
                            attr::OBJPATH => {
                                let p = a.payload.strip_suffix(&[0]).unwrap_or(a.payload);
                                info.path = String::from_utf8_lossy(p).into_owned();
                            }
                            attr::OBJID => info.id = u32_payload(a.payload).unwrap_or(0),
                            attr::OBJTYPE => info.type_id = u32_payload(a.payload).unwrap_or(0),
                            _ => {}
                        }
                    }
                    found.push(info);
                }
                MessageType::Status => {
                    let status = status_of(&msg)?;
                    return if status == 0 {
                        Ok(found)
                    } else {
                        Err(UbusError::Status(status))
                    };
                }
                _ => {}
            }
        }
    }

    /// Calls `method` on the object at `path` with `args`, returning the merged reply tables.
    pub fn call(
        &mut self,
        path: &str,
        method: &str,
        args: &Map<String, Value>,
    ) -> Result<Map<String, Value>, UbusError> {
        let objects = self.lookup(path)?;
        let object = objects
            .into_iter()
            .find(|o| o.path == path)
            .ok_or_else(|| UbusError::NotFound(path.to_string()))?;
        let attrs = vec![
            codec::attr_encode(attr::OBJID, &object.id.to_be_bytes()),
            codec::attr_encode(attr::METHOD, &cstring(method)),
            codec::attr_encode(attr::DATA, &codec::blobmsg_encode_table(args)),
        ];
        let seq = self.send(MessageType::Invoke, object.id, &attrs)?;
        let mut reply = Map::new();
        loop {
            let msg = self.read_message()?;
            if msg.seq != seq {
                continue;
            }
            match msg.kind {
                MessageType::Data => {
                    for a in codec::attrs_decode(&msg.body)? {
                        if a.id == attr::DATA {
                            reply.extend(codec::blobmsg_decode_table(a.payload)?);
                        }
                    }
                }
                MessageType::Status => {
                    let status = status_of(&msg)?;
                    return if status == 0 {
                        Ok(reply)
                    } else {
                        Err(UbusError::Status(status))
                    };
                }
                _ => {}
            }
        }
    }
}

fn status_of(msg: &Message) -> Result<u32, UbusError> {
    for a in codec::attrs_decode(&msg.body)? {
        if a.id == attr::STATUS {
            return u32_payload(a.payload).ok_or(UbusError::Protocol("status without value"));
        }
    }
    Err(UbusError::Protocol("STATUS without status attribute"))
}
