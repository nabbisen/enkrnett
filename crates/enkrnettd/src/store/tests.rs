//! ENK-BASE-005, ENK-CFG-006, ENK-OWN-018, ENK-OWN-020.

use std::sync::atomic::{AtomicUsize, Ordering};

use enkrnett_model::settings::{GuestMode, GuestSettings, ManagementSettings, Settings};

use super::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> TempDir {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let p =
            std::env::temp_dir().join(format!("enkrnettd-store-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        TempDir(p)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn settings() -> Settings {
    Settings {
        version: 1,
        country: "JP".into(),
        guest: GuestSettings {
            name: "Cafe".into(),
            mode: GuestMode::OpenToAll,
            enabled: true,
        },
        management: ManagementSettings {
            name: "Enkrnett-3F2A".into(),
        },
    }
}

#[test]
fn a_fresh_store_has_an_identifier_and_no_owner() {
    let dir = TempDir::new();
    let s = Store::open(&dir.0, || "0123456789abcdef".into()).unwrap();
    assert_eq!(s.data.device_id, "0123456789abcdef");
    assert!(s.data.owner.is_none());
    assert_eq!(s.data.format, STORAGE_FORMAT);
    assert!(s.path().exists());
}

#[test]
fn save_and_reopen() {
    let dir = TempDir::new();
    let mut s = Store::open(&dir.0, || "a".repeat(16)).unwrap();
    s.data.owner = Some(OwnerRecord {
        password_hash: "$argon2id$x".into(),
        management_name: "Enkrnett-3F2A".into(),
        management_key: "abcde-fghjk-mnpqr-stuvw".into(),
        settings: settings(),
    });
    s.save().unwrap();
    assert!(
        !dir.0.join("state.json.tmp").exists(),
        "temporary file renamed away"
    );
    let again = Store::open(&dir.0, || "b".repeat(16)).unwrap();
    assert_eq!(again.data, s.data);
}

#[test]
fn erase_removes_everything_and_makes_a_new_identifier() {
    let dir = TempDir::new();
    let mut s = Store::open(&dir.0, || "first-id-00000000".into()).unwrap();
    s.data.owner = Some(OwnerRecord {
        password_hash: "h".into(),
        management_name: "m".into(),
        management_key: "k".into(),
        settings: settings(),
    });
    s.save().unwrap();
    fs::write(dir.0.join("leftover.log"), b"x").unwrap();
    s.erase(|| "second-id-0000000".into()).unwrap();
    assert!(s.data.owner.is_none());
    assert_eq!(s.data.device_id, "second-id-0000000");
    assert!(!dir.0.join("leftover.log").exists());
    let content = fs::read_to_string(s.path()).unwrap();
    assert!(
        !content.contains("\"k\""),
        "nothing of the old record remains"
    );
}

#[test]
fn a_newer_format_is_refused() {
    let dir = TempDir::new();
    fs::create_dir_all(&dir.0).unwrap();
    fs::write(
        dir.0.join("state.json"),
        format!(
            r#"{{"format":{},"device_id":"x","owner":null}}"#,
            STORAGE_FORMAT + 1
        ),
    )
    .unwrap();
    let err = Store::open(&dir.0, || "x".into()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn an_older_document_without_new_fields_still_loads() {
    let dir = TempDir::new();
    fs::create_dir_all(&dir.0).unwrap();
    fs::write(
        dir.0.join("state.json"),
        r#"{"format":1,"device_id":"0123456789abcdef","owner":null}"#,
    )
    .unwrap();
    let s = Store::open(&dir.0, || "x".into()).unwrap();
    assert_eq!(s.data.index_sequence, 0);
}
