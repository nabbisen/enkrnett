//! `enkrnett-release`: the release tool of chapter 11 (ENK-RLS-004 to 007). It makes and uses
//! signify-compatible keys, signs and verifies files, and writes the release index. It runs on
//! the signing host (the secret key never reaches a CI runner, ENK-RLS-007) and on a
//! workstation for the public operations and the tests.
//!
//! ```text
//! enkrnett-release keygen [-c COMMENT] NAME                 NAME.sec (mode 0600) and NAME.pub
//! enkrnett-release sign KEY.sec FILE [SIGNATURE]            detached signature, FILE.sig by default
//! enkrnett-release verify FILE [SIGNATURE] --key KEY.pub... exit 0 when one of the keys verifies it
//! enkrnett-release sha512 FILE                              lower-case hex
//! enkrnett-release index --sequence N [--out FILE] \
//!     --release PROFILE VERSION BASELINE FROM_BASELINE IMAGE MIN_FREE_KIB NOTES_URL INSTALL_SECONDS ...
//! ```
//!
//! The secret key is written unencrypted, as `signify -G -n` does: the signing host keeps it
//! offline (RFC 008). A key made here verifies with OpenBSD `signify -V`, and the other way
//! round.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::exit;

use enkrnett_model::release::{ReleaseEntry, ReleaseIndex};
use enkrnett_model::signify::{self, PublicKey, SecretKey, Signature};
use rand_core::{OsRng, RngCore};

fn usage() -> ! {
    eprintln!(
        "usage: enkrnett-release keygen [-c COMMENT] NAME\n       \
         enkrnett-release sign KEY.sec FILE [SIGNATURE]\n       \
         enkrnett-release verify FILE [SIGNATURE] --key KEY.pub...\n       \
         enkrnett-release sha512 FILE\n       \
         enkrnett-release index --sequence N [--out FILE] --release PROFILE VERSION BASELINE FROM_BASELINE IMAGE MIN_FREE_KIB NOTES_URL INSTALL_SECONDS ..."
    );
    exit(2)
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("enkrnett-release: {message}");
    exit(1)
}

fn read(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| fail(format!("read {path}: {e}")))
}

fn read_text(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| fail(format!("read {path}: {e}")))
}

fn write(path: &str, bytes: &[u8], mode: u32) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .unwrap_or_else(|e| fail(format!("create {path}: {e}")));
    f.write_all(bytes)
        .unwrap_or_else(|e| fail(format!("write {path}: {e}")));
}

fn keygen(args: &[String]) {
    let mut comment: Option<String> = None;
    let mut name: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                comment = Some(args.get(i + 1).cloned().unwrap_or_else(|| usage()));
                i += 2;
            }
            a if name.is_none() => {
                name = Some(a);
                i += 1;
            }
            _ => usage(),
        }
    }
    let name = name.unwrap_or_else(|| usage());
    let comment = comment.unwrap_or_else(|| format!("{name} release key"));
    let mut seed = [0u8; 32];
    let mut keynum = [0u8; 8];
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut seed);
    OsRng.fill_bytes(&mut keynum);
    OsRng.fill_bytes(&mut salt);
    let secret = signify::key_pair(seed, keynum, salt);
    write(
        &format!("{name}.sec"),
        secret.to_text(&format!("{comment} secret key")).as_bytes(),
        0o600,
    );
    write(
        &format!("{name}.pub"),
        secret
            .public_key()
            .to_text(&format!("{comment} public key"))
            .as_bytes(),
        0o644,
    );
    println!(
        "key {} written to {name}.sec (keep it on the signing host) and {name}.pub",
        signify::hex(&keynum)
    );
}

fn sign(args: &[String]) {
    let [key, file, rest @ ..] = args else {
        usage()
    };
    let out = rest
        .first()
        .cloned()
        .unwrap_or_else(|| format!("{file}.sig"));
    if rest.len() > 1 {
        usage();
    }
    let secret = SecretKey::parse(&read_text(key)).unwrap_or_else(|e| fail(e));
    let message = read(file);
    let signature = secret.sign(&message);
    let key_name = Path::new(key)
        .file_name()
        .map(|n| n.to_string_lossy().replace(".sec", ".pub"))
        .unwrap_or_else(|| "key.pub".into());
    let _ = fs::remove_file(&out);
    write(
        &out,
        signature
            .to_text(&format!("verify with {key_name}"))
            .as_bytes(),
        0o644,
    );
    println!(
        "signed {file} with key {} -> {out}",
        signify::hex(&secret.keynum)
    );
}

fn verify(args: &[String]) {
    let mut keys: Vec<PublicKey> = Vec::new();
    let mut files: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--key" {
            let path = args.get(i + 1).unwrap_or_else(|| usage());
            keys.push(
                PublicKey::parse(&read_text(path)).unwrap_or_else(|e| fail(format!("{path}: {e}"))),
            );
            i += 2;
        } else {
            files.push(&args[i]);
            i += 1;
        }
    }
    let (file, sig) = match files.as_slice() {
        [f] => (*f, format!("{f}.sig")),
        [f, s] => (*f, s.to_string()),
        _ => usage(),
    };
    if keys.is_empty() {
        usage();
    }
    let signature =
        Signature::parse(&read_text(&sig)).unwrap_or_else(|e| fail(format!("{sig}: {e}")));
    match signify::verify(&keys, &read(file), &signature) {
        Ok(k) => println!(
            "{file}: signature verified with key {}",
            signify::hex(&k.keynum)
        ),
        Err(e) => fail(format!("{file}: {e}")),
    }
}

fn sha512(args: &[String]) {
    let [file] = args else { usage() };
    println!("{}  {file}", signify::hex(&signify::sha512(&read(file))));
}

fn index(args: &[String]) {
    let mut sequence: Option<u64> = None;
    let mut out: Option<String> = None;
    let mut releases: Vec<ReleaseEntry> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--sequence" => {
                sequence = args.get(i + 1).and_then(|s| s.parse().ok());
                if sequence.is_none() {
                    usage();
                }
                i += 2;
            }
            "--out" => {
                out = Some(args.get(i + 1).cloned().unwrap_or_else(|| usage()));
                i += 2;
            }
            "--release" => {
                let f = &args[i + 1..];
                if f.len() < 8 {
                    usage();
                }
                let image = &f[4];
                let bytes = read(image);
                let file = Path::new(image)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| usage());
                releases.push(ReleaseEntry {
                    profile: f[0].clone(),
                    version: f[1].clone(),
                    baseline: f[2].clone(),
                    from_baseline: f[3].clone(),
                    file,
                    size: bytes.len() as u64,
                    sha512: signify::hex(&signify::sha512(&bytes)),
                    min_free_kib: f[5].parse().unwrap_or_else(|_| usage()),
                    notes_url: f[6].clone(),
                    install_seconds: f[7].parse().unwrap_or_else(|_| usage()),
                });
                i += 9;
            }
            _ => usage(),
        }
    }
    let index = ReleaseIndex {
        format: enkrnett_model::release::INDEX_FORMAT,
        sequence: sequence.unwrap_or_else(|| usage()),
        releases,
    };
    let mut json = serde_json::to_vec_pretty(&index).expect("serialisable");
    json.push(b'\n');
    // What a device will do with it: the same parse, the same refusals.
    if let Err(e) = ReleaseIndex::parse(&json) {
        fail(format!("the index would be refused by a device: {e}"));
    }
    match out {
        Some(path) => {
            let _ = fs::remove_file(&path);
            write(&path, &json, 0o644);
            println!(
                "index with sequence {} and {} release(s) -> {path} (sign it next)",
                index.sequence,
                index.releases.len()
            );
        }
        None => {
            let _ = std::io::stdout().write_all(&json);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        usage()
    };
    match command.as_str() {
        "keygen" => keygen(rest),
        "sign" => sign(rest),
        "verify" => verify(rest),
        "sha512" => sha512(rest),
        "index" => index(rest),
        _ => usage(),
    }
}
