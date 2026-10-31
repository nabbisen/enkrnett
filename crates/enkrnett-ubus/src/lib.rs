//! A small ubus client for the Enkrnett daemon (RFC 010 ENK-FOOT-014: reach `ubusd` through an
//! own client on the unix socket rather than by spawning the `ubus` command).
//!
//! [`codec`] is the wire format, pure and tested against byte vectors. [`client`] speaks it on
//! `ubusd`'s socket: `lookup` an object path and `call` a method with a JSON argument table.
//! Only what the daemon needs is implemented; nothing is published that `ubusd` does not
//! already export.

pub mod client;
pub mod codec;
