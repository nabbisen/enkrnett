//! Strict JSON (ENK-SEC-020): no duplicate keys, nesting at most 8, numbers that are integers
//! within 64 bits. Checked before the typed parse, over the same bytes.

use std::collections::HashSet;
use std::fmt;

use serde::de::{DeserializeSeed, Deserializer, Error as _, MapAccess, SeqAccess, Visitor};

/// Deepest nesting of arrays and objects accepted.
pub const MAX_DEPTH: usize = 8;

/// Checks the bytes; the message names the rule that failed.
pub fn check_strict(bytes: &[u8]) -> Result<(), &'static str> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    Node { depth: 0 }.deserialize(&mut de).map_err(|e| {
        let text = e.to_string();
        if text.contains("duplicate key") {
            "duplicate key"
        } else if text.contains("too deep") {
            "nesting deeper than 8"
        } else if text.contains("not an integer") {
            "numbers must be integers within 64 bits"
        } else {
            "body is not valid JSON"
        }
    })?;
    de.end()
        .map_err(|_| "trailing characters after the JSON value")
}

struct Node {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Node {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Node {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E>(self, _: bool) -> Result<(), E> {
        Ok(())
    }

    fn visit_i64<E>(self, _: i64) -> Result<(), E> {
        Ok(())
    }

    fn visit_u64<E>(self, _: u64) -> Result<(), E> {
        Ok(())
    }

    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
        Err(E::custom("not an integer"))
    }

    fn visit_str<E>(self, _: &str) -> Result<(), E> {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_none<E>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(A::Error::custom("too deep"));
        }
        while seq
            .next_element_seed(Node {
                depth: self.depth + 1,
            })?
            .is_some()
        {}
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(A::Error::custom("too deep"));
        }
        let mut keys: HashSet<String> = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(A::Error::custom("duplicate key"));
            }
            map.next_value_seed(Node {
                depth: self.depth + 1,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
