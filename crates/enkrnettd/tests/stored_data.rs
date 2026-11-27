//! ENK-BASE-005: the current daemon reads what every earlier release wrote. The documents in
//! `tests/fixtures/stored/<version>/` were written by the daemons of the tagged releases
//! (`fixtures/stored/README.md`); the test starts today's daemon over a copy of each, as the first
//! boot of an update does (ENK-UPD-004, 005), and checks that the owner, the settings and the
//! identifier are there and usable. The daemon under test reports the next version, so that the
//! current release's own fixture is read as a predecessor too.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{get, login, post, start_on_configured};
use enkrnett_model::STORAGE_FORMAT;
use serde_json::Value;

const RELEASES: &[&str] = &["0.1.0", "0.2.0", "0.3.0"];

fn fixture(version: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stored")
        .join(version)
}

fn copy_of(version: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("enkrnettd-stored-{}-{version}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for entry in fs::read_dir(fixture(version)).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), dir.join(entry.file_name())).unwrap();
    }
    dir
}

fn stored(dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(dir.join("state.json")).unwrap()).unwrap()
}

#[test]
fn the_documents_of_every_earlier_release_are_read_and_served() {
    for version in RELEASES {
        let dir = copy_of(version);
        let before = stored(&dir);
        assert_eq!(before["format"], 1, "{version}: the fixture is format 1");
        let device_id = before["device_id"].as_str().unwrap().to_string();

        // Today's daemon over the old directory: the first boot after an update (ENK-UPD-005).
        let ts = start_on_configured(&dir, |c| c.version = "0.4.0".into(), |_| {});
        let greeting = get(&ts, "", None).json();
        assert_eq!(greeting["state"], "CLAIMED", "{version}: {greeting}");

        // The owner's password hash verifies; the settings and the identifier are as written.
        let token = login(&ts);
        let status = get(&ts, "status", Some(&token)).json();
        assert_eq!(
            status["settings"]["guest"]["name"], "Cafe Example",
            "{version}"
        );
        assert_eq!(
            status["settings"]["guest"]["mode"], "encrypted_only",
            "{version}"
        );
        assert_eq!(status["settings"]["country"], "JP", "{version}");
        let device = get(&ts, "device", Some(&token)).json();
        assert_eq!(device["device_id"], device_id.as_str(), "{version}");
        assert_eq!(device["version"]["enkrnett"], "0.4.0", "{version}");

        // What the old release never wrote defaults: no update check switch, no sequence beyond 0.
        let update = get(&ts, "update", Some(&token)).json();
        assert_eq!(update["state"], "idle", "{version}");
        assert!(update["last_result"].is_null(), "{version}: {update}");

        // A change rewrites the document in the current format and keeps what it carried.
        let r = post(
            &ts,
            "settings",
            Some(&token),
            r#"{"version":2,"guest":{"name":"Cafe Example 2","mode":"encrypted_only","enabled":true},"management":{"name":"Enkrnett-3F2A"}}"#,
        );
        assert!(
            r.status == 200 || r.status == 202,
            "{version}: {} {}",
            r.status,
            r.body
        );
        common::wait_for_change_state(&ts, &token, "settled");
        let after = stored(&dir);
        assert_eq!(after["format"], STORAGE_FORMAT, "{version}");
        assert_eq!(after["device_id"], device_id.as_str(), "{version}");
        assert_eq!(
            after["owner"]["settings"]["guest"]["name"], "Cafe Example 2",
            "{version}"
        );
        assert_eq!(
            after["owner"]["password_hash"], before["owner"]["password_hash"],
            "{version}: the password hash is carried, never rewritten"
        );
        assert_eq!(
            after["owner"]["management_key"], before["owner"]["management_key"],
            "{version}: the management key is carried"
        );
        drop(ts);
    }
}

#[test]
fn every_release_in_the_notes_has_a_fixture() {
    // The release pages exist for exactly the tagged releases; each one must have left its data here.
    let releases = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/src/releases");
    for entry in fs::read_dir(releases).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if name == "index.md" || !name.ends_with(".md") {
            continue;
        }
        let version = name.trim_end_matches(".md");
        assert!(
            fixture(version).join("state.json").is_file(),
            "release {version} has notes but no stored-data fixture (ENK-BASE-005)"
        );
    }
}
