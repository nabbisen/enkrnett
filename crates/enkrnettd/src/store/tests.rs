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
fn a_document_from_release_0_2_0_without_the_update_fields_is_read() {
    // ENK-BASE-005: fields added after 0.2.0 default; the format number did not change because
    // a 0.2.0 daemon would read a newer document correctly too (it ignores what it does not know).
    let dir = TempDir::new();
    fs::create_dir_all(&dir.0).unwrap();
    fs::write(
        dir.0.join(FILE_NAME),
        br#"{"format":1,"device_id":"ABCD-1234","owner":null}"#,
    )
    .unwrap();
    let store = Store::open(&dir.0, || "X".into()).unwrap();
    assert_eq!(store.data.device_id, "ABCD-1234");
    assert_eq!(store.data.index_sequence, 0);
    assert_eq!(store.data.update_checks, None);
    assert_eq!(store.data.last_auto_check, None);
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

#[test]
fn the_pending_marker_is_written_read_and_removed() {
    // ENK-CFG-006, ENK-CFG-009: the marker survives a restart; removing it twice is harmless.
    let dir = TempDir::new();
    let store = Store::open(&dir.0, || "dev".into()).unwrap();
    assert!(store.read_pending().is_none());
    assert!(!store.has_pending());
    let marker = PendingMarker {
        format: STORAGE_FORMAT,
        id: 3,
        kind: ChangeKind::Settings,
        from_version: 1,
        to_version: 2,
        needs_confirmation: false,
        previous: OwnerRecord {
            password_hash: "h".into(),
            management_name: "Enkrnett-3F2A".into(),
            management_key: "k".into(),
            settings: settings(),
        },
    };
    store.write_pending(&marker).unwrap();
    assert!(store.has_pending());
    assert!(!dir.0.join("pending.json.tmp").exists());
    let again = Store::open(&dir.0, || "other".into()).unwrap();
    assert_eq!(again.read_pending().unwrap().unwrap(), marker);
    again.remove_pending().unwrap();
    again.remove_pending().unwrap();
    assert!(again.read_pending().is_none());
}

#[test]
fn an_unreadable_marker_still_counts_as_pending() {
    let dir = TempDir::new();
    let store = Store::open(&dir.0, || "dev".into()).unwrap();
    std::fs::write(dir.0.join("pending.json"), b"{ not json").unwrap();
    assert!(matches!(store.read_pending(), Some(Err(_))));
    store.remove_pending().unwrap();
    assert!(store.read_pending().is_none());
}

#[test]
fn stored_files_are_readable_by_the_daemon_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new();
    let mut s = Store::open(&dir.0, || "a".repeat(16)).unwrap();
    s.data.owner = None;
    s.save().unwrap();
    let mode = std::fs::metadata(dir.0.join("state.json"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn the_update_marker_records_installing_then_the_outcome_and_survives_a_restart() {
    // ENK-UPD-005: written before the image is handed over; the first boot reads it and turns it
    // into the outcome, which stays until the next attempt. Erase removes it with the rest.
    let dir = TempDir::new();
    let store = Store::open(&dir.0, || "dev".into()).unwrap();
    assert!(store.read_update().is_none());
    let marker = UpdateMarker::installing("0.2.0", "0.3.0", "25.12", true);
    assert_eq!(marker.last_result(), None);
    store.write_update(&marker).unwrap();
    assert!(!dir.0.join("update.json.tmp").exists());
    let again = Store::open(&dir.0, || "other".into()).unwrap();
    assert_eq!(again.read_update(), Some(marker.clone()));
    let done = marker.clone().done();
    assert_eq!(done.stage, UpdateStage::Done);
    assert_eq!(
        done.last_result(),
        Some(LastResult {
            result: UpdateOutcome::Done,
            reason: None
        })
    );
    let failed = marker.failed(UpdateFailure::HealthCheckFailed);
    again.write_update(&failed).unwrap();
    assert_eq!(
        again.read_update().unwrap().last_result(),
        Some(LastResult {
            result: UpdateOutcome::Failed,
            reason: Some(UpdateFailure::HealthCheckFailed)
        })
    );
    // A marker from a newer format, or not a marker at all, counts as none.
    fs::write(
        dir.0.join("update.json"),
        br#"{"format": 99, "stage": "done"}"#,
    )
    .unwrap();
    assert!(again.read_update().is_none());
    fs::write(dir.0.join("update.json"), b"{ not json").unwrap();
    assert!(again.read_update().is_none());
    again.remove_update().unwrap();
    again.remove_update().unwrap();
    assert!(!dir.0.join("update.json").exists());
}
