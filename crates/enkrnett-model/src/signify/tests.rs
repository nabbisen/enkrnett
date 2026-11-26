//! ENK-RLS-004, ENK-RLS-005, ENK-UPD-003: the signify formats and Ed25519 verification,
//! including the hostile inputs of chapter 12.

use super::*;

fn key(n: u8) -> SecretKey {
    key_pair([n; 32], [n, 1, 2, 3, 4, 5, 6, 7], [9; 16])
}

#[test]
fn a_signature_verifies_under_its_key_and_under_no_other() {
    let a = key(1);
    let b = key(2);
    let msg = b"the release index";
    let sig = a.sign(msg);
    let keys = [b.public_key(), a.public_key()];
    assert_eq!(verify(&keys, msg, &sig), Ok(&a.public_key()));
    assert_eq!(
        verify(&[b.public_key()], msg, &sig),
        Err(SignifyError::UnknownKey)
    );
    assert_eq!(
        verify(&keys, b"the release index.", &sig),
        Err(SignifyError::Signature)
    );
    let mut forged = sig.clone();
    forged.sig[10] ^= 1;
    assert_eq!(verify(&keys, msg, &forged), Err(SignifyError::Signature));
    // Same key number, other key: the number is a hint, never the proof.
    let mut impostor = b.public_key();
    impostor.keynum = a.keynum;
    assert_eq!(verify(&[impostor], msg, &sig), Err(SignifyError::Signature));
}

#[test]
fn the_text_forms_round_trip_and_look_like_signify_files() {
    let k = key(3);
    let pk_text = k.public_key().to_text("enkrnett test public key");
    assert!(pk_text.starts_with("untrusted comment: enkrnett test public key\n"));
    assert_eq!(pk_text.lines().count(), 2);
    assert_eq!(PublicKey::parse(&pk_text), Ok(k.public_key()));
    let sig = k.sign(b"m");
    let sig_text = sig.to_text("verify with test.pub");
    assert_eq!(Signature::parse(&sig_text), Ok(sig));
    let sk_text = k.to_text("enkrnett test secret key");
    assert_eq!(SecretKey::parse(&sk_text), Ok(k.clone()));
    // Lengths as signify writes them: 42, 74 and 104 bytes before Base64.
    assert_eq!(pk_text.lines().nth(1).unwrap().len(), 56);
    assert_eq!(sig_text.lines().nth(1).unwrap().len(), 100);
    assert_eq!(sk_text.lines().nth(1).unwrap().len(), 140);
}

#[test]
fn an_ed25519_signature_from_another_implementation_verifies() {
    // Key and signature over the text "hello\n" made with OpenSSL 3.5 (`openssl genpkey
    // -algorithm ed25519`, `openssl pkeyutl -sign -rawin`), then laid out as signify does:
    // "Ed", the key number 0102030405060708, the raw key or signature, Base64. No signify
    // binary was at hand; the layout is transcribed from signify.c. If signify's bytes differ,
    // this test is where it shows.
    let pk = "untrusted comment: openssl-made key\n\
              RWQBAgMEBQYHCEvLeHBYuW9P0nNmtA+zZXTxM8Fn7rQdMbqcQzFF6sQa\n";
    let sig = "untrusted comment: verify with key.pub\n\
               RWQBAgMEBQYHCFrHbwT0cf2aZuBXBtGuxyL3wuEIXFFInmpV1bx38aEZwLrqg3o7PgYCqw1ucn7W29SY2ad3VeUsZqjIrkFd+Q8=\n";
    let pk = PublicKey::parse(pk).unwrap();
    let sig = Signature::parse(sig).unwrap();
    assert_eq!(pk.keynum, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(verify(std::slice::from_ref(&pk), b"hello\n", &sig), Ok(&pk));
    assert_eq!(
        verify(std::slice::from_ref(&pk), b"hello", &sig),
        Err(SignifyError::Signature)
    );
}

#[test]
fn hostile_files_are_refused_with_a_reason() {
    let k = key(4);
    let pk = k.public_key().to_text("c");
    let sig = k.sign(b"m").to_text("c");
    assert_eq!(
        PublicKey::parse(""),
        Err(SignifyError::Format("missing comment line"))
    );
    assert_eq!(
        PublicKey::parse("comment: x\nAAAA\n"),
        Err(SignifyError::Format("missing comment line"))
    );
    assert_eq!(
        PublicKey::parse("untrusted comment: x"),
        Err(SignifyError::Format("missing data line"))
    );
    assert_eq!(
        PublicKey::parse("untrusted comment: x\n!!!!\n"),
        Err(SignifyError::Format("base64"))
    );
    assert_eq!(
        PublicKey::parse("untrusted comment: x\nAAAA\n"),
        Err(SignifyError::Format("public key length"))
    );
    // A signature file given where a key is expected, and the other way round.
    assert_eq!(
        PublicKey::parse(&sig),
        Err(SignifyError::Format("public key length"))
    );
    assert_eq!(
        Signature::parse(&pk),
        Err(SignifyError::Format("signature length"))
    );
    // Wrong algorithm bytes.
    let mut body = vec![b'X', b'd'];
    body.extend_from_slice(&[0; 40]);
    assert_eq!(
        PublicKey::parse(&to_text("x", &body)),
        Err(SignifyError::Format("public key algorithm"))
    );
    // Extra content after the two lines.
    assert_eq!(
        PublicKey::parse(&format!("{pk}trailer\n")),
        Err(SignifyError::Format("unexpected extra lines"))
    );
    assert!(PublicKey::parse(&format!("{pk}\n\n")).is_ok());
    // Far too long.
    let long = format!("untrusted comment: {}\nAAAA\n", "x".repeat(MAX_TEXT));
    assert_eq!(
        PublicKey::parse(&long),
        Err(SignifyError::Format("file too long"))
    );
}

#[test]
fn a_passphrase_protected_or_corrupt_secret_key_is_refused() {
    let k = key(5);
    let text = k.to_text("c");
    let (_, mut bytes) = parse_file(&text).unwrap();
    bytes[7] = 42; // kdf rounds
    assert_eq!(
        SecretKey::parse(&to_text("c", &bytes)),
        Err(SignifyError::Encrypted)
    );
    let (_, mut bytes) = parse_file(&text).unwrap();
    bytes[50] ^= 1; // a secret byte: the checksum no longer matches
    assert_eq!(
        SecretKey::parse(&to_text("c", &bytes)),
        Err(SignifyError::Format("secret key checksum"))
    );
}

#[test]
fn sha512_and_hex() {
    assert_eq!(
        hex(&sha512(b"abc")),
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
    );
}
