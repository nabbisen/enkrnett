//! OpenBSD `signify` formats (public key, signature, unencrypted secret key), verified with
//! Ed25519 (chapter 11 §1–2: ENK-RLS-004, ENK-RLS-005, ENK-UPD-003). Only what the device and
//! the release tool need: no passphrase-protected secret keys (the signing host keeps its key as
//! `signify -G -n` writes it), no embedded signatures. Interoperable with `signify(1)`: a key
//! made by either side verifies the other's signatures, because both sign the file's bytes with
//! plain Ed25519.
//!
//! A file is two lines: `untrusted comment: …` and one Base64 line. Decoded, a public key is
//! `Ed` + 8 bytes key number + 32 bytes key; a signature is `Ed` + key number + 64 bytes; a
//! secret key is `Ed` + `BK` + rounds (4 bytes, big-endian, 0 when unencrypted) + 16 bytes salt
//! + 8 bytes checksum (the first 8 of SHA-512 of the secret) + key number + 64 bytes secret.

use ct_codecs::{Base64, Decoder, Encoder};
use ed25519_compact as ed;

pub const COMMENT_HEADER: &str = "untrusted comment: ";
/// A key or signature file: two short lines. Anything longer is refused before decoding.
pub const MAX_TEXT: usize = 2048;
pub const KEYNUM_BYTES: usize = 8;
const PKALG: &[u8; 2] = b"Ed";
const KDFALG: &[u8; 2] = b"BK";
const PUBKEY_LEN: usize = 2 + KEYNUM_BYTES + 32;
const SIG_LEN: usize = 2 + KEYNUM_BYTES + 64;
const SECKEY_LEN: usize = 2 + 2 + 4 + 16 + 8 + KEYNUM_BYTES + 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignifyError {
    /// Not a signify file of the expected kind; the word says what was wrong.
    Format(&'static str),
    /// The signature names a key number none of the given keys has.
    UnknownKey,
    /// The secret key is protected by a passphrase, which this implementation does not read.
    Encrypted,
    /// The signature does not verify under the named key.
    Signature,
}

impl std::fmt::Display for SignifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignifyError::Format(what) => write!(f, "signify format: {what}"),
            SignifyError::UnknownKey => write!(f, "signify: no key with that number"),
            SignifyError::Encrypted => write!(f, "signify: the secret key is passphrase-protected"),
            SignifyError::Signature => write!(f, "signify: the signature does not verify"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    pub keynum: [u8; KEYNUM_BYTES],
    pub key: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub keynum: [u8; KEYNUM_BYTES],
    pub sig: [u8; 64],
}

/// An unencrypted signify secret key (what `signify -G -n` writes).
#[derive(Clone, PartialEq, Eq)]
pub struct SecretKey {
    pub keynum: [u8; KEYNUM_BYTES],
    pub salt: [u8; 16],
    secret: [u8; 64],
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretKey(keynum {})", hex(&self.keynum))
    }
}

/// Splits a signify file into its comment and the decoded body.
fn parse_file(text: &str) -> Result<(&str, Vec<u8>), SignifyError> {
    if text.len() > MAX_TEXT {
        return Err(SignifyError::Format("file too long"));
    }
    let mut lines = text.split('\n');
    let comment = lines
        .next()
        .and_then(|l| l.strip_prefix(COMMENT_HEADER))
        .ok_or(SignifyError::Format("missing comment line"))?;
    let body = lines
        .next()
        .ok_or(SignifyError::Format("missing data line"))?
        .trim_end_matches('\r');
    if lines.any(|l| !l.trim().is_empty()) {
        return Err(SignifyError::Format("unexpected extra lines"));
    }
    let bytes = Base64::decode_to_vec(body, None).map_err(|_| SignifyError::Format("base64"))?;
    Ok((comment, bytes))
}

fn to_text(comment: &str, body: &[u8]) -> String {
    let b64 = Base64::encode_to_string(body).expect("encoding cannot fail");
    format!("{COMMENT_HEADER}{comment}\n{b64}\n")
}

fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    out
}

impl PublicKey {
    pub fn parse(text: &str) -> Result<PublicKey, SignifyError> {
        let (_, bytes) = parse_file(text)?;
        if bytes.len() != PUBKEY_LEN {
            return Err(SignifyError::Format("public key length"));
        }
        if &bytes[0..2] != PKALG {
            return Err(SignifyError::Format("public key algorithm"));
        }
        Ok(PublicKey {
            keynum: array(&bytes[2..10]),
            key: array(&bytes[10..42]),
        })
    }

    pub fn to_text(&self, comment: &str) -> String {
        let mut body = Vec::with_capacity(PUBKEY_LEN);
        body.extend_from_slice(PKALG);
        body.extend_from_slice(&self.keynum);
        body.extend_from_slice(&self.key);
        to_text(comment, &body)
    }
}

impl Signature {
    pub fn parse(text: &str) -> Result<Signature, SignifyError> {
        let (_, bytes) = parse_file(text)?;
        if bytes.len() != SIG_LEN {
            return Err(SignifyError::Format("signature length"));
        }
        if &bytes[0..2] != PKALG {
            return Err(SignifyError::Format("signature algorithm"));
        }
        Ok(Signature {
            keynum: array(&bytes[2..10]),
            sig: array(&bytes[10..74]),
        })
    }

    pub fn to_text(&self, comment: &str) -> String {
        let mut body = Vec::with_capacity(SIG_LEN);
        body.extend_from_slice(PKALG);
        body.extend_from_slice(&self.keynum);
        body.extend_from_slice(&self.sig);
        to_text(comment, &body)
    }
}

impl SecretKey {
    pub fn parse(text: &str) -> Result<SecretKey, SignifyError> {
        let (_, bytes) = parse_file(text)?;
        if bytes.len() != SECKEY_LEN {
            return Err(SignifyError::Format("secret key length"));
        }
        if &bytes[0..2] != PKALG {
            return Err(SignifyError::Format("secret key algorithm"));
        }
        if &bytes[2..4] != KDFALG {
            return Err(SignifyError::Format("secret key kdf"));
        }
        if bytes[4..8] != [0, 0, 0, 0] {
            return Err(SignifyError::Encrypted);
        }
        let secret: [u8; 64] = array(&bytes[40..104]);
        if bytes[24..32] != sha512(&secret)[..8] {
            return Err(SignifyError::Format("secret key checksum"));
        }
        Ok(SecretKey {
            keynum: array(&bytes[32..40]),
            salt: array(&bytes[8..24]),
            secret,
        })
    }

    pub fn to_text(&self, comment: &str) -> String {
        let mut body = Vec::with_capacity(SECKEY_LEN);
        body.extend_from_slice(PKALG);
        body.extend_from_slice(KDFALG);
        body.extend_from_slice(&[0, 0, 0, 0]);
        body.extend_from_slice(&self.salt);
        body.extend_from_slice(&sha512(&self.secret)[..8]);
        body.extend_from_slice(&self.keynum);
        body.extend_from_slice(&self.secret);
        to_text(comment, &body)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            keynum: self.keynum,
            key: array(&self.secret[32..64]),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let sk = ed::SecretKey::from_slice(&self.secret).expect("64-byte secret");
        Signature {
            keynum: self.keynum,
            sig: *sk.sign(message, None),
        }
    }
}

/// A new key pair from 32 random bytes and an 8-byte key number (the caller brings the
/// randomness: this crate does no I/O).
pub fn key_pair(seed: [u8; 32], keynum: [u8; KEYNUM_BYTES], salt: [u8; 16]) -> SecretKey {
    let kp = ed::KeyPair::from_seed(ed::Seed::new(seed));
    SecretKey {
        keynum,
        salt,
        secret: *kp.sk,
    }
}

/// Verifies `message` against `signature` with the key whose number the signature names, and
/// returns that key. ENK-RLS-005: the device holds the current key and one designated next key,
/// so a signature by either verifies.
pub fn verify<'k>(
    keys: &'k [PublicKey],
    message: &[u8],
    signature: &Signature,
) -> Result<&'k PublicKey, SignifyError> {
    let key = keys
        .iter()
        .find(|k| k.keynum == signature.keynum)
        .ok_or(SignifyError::UnknownKey)?;
    let pk = ed::PublicKey::new(key.key);
    pk.verify(message, &ed::Signature::new(signature.sig))
        .map_err(|_| SignifyError::Signature)?;
    Ok(key)
}

pub fn sha512(bytes: &[u8]) -> [u8; 64] {
    ed::sha512::Hash::hash(bytes)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests;
