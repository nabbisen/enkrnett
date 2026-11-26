//! The update check against the mock platform (chapter 11 §2–3): ENK-UPD-001, ENK-UPD-008,
//! ENK-UPD-009, ENK-RLS-005, ENK-RLS-006, ENK-API-012.

mod common;

use common::*;
use enkrnett_model::release::{ReleaseEntry, ReleaseIndex};
use enkrnett_model::signify::{SecretKey, key_pair};
use enkrnettd::platform::{FetchError, MockPlatform};

const URL: &str = "https://updates.example/index.json";

fn key(n: u8) -> SecretKey {
    key_pair([n; 32], [n; 8], [0; 16])
}

fn entry(version: &str, from: &str, to: &str) -> ReleaseEntry {
    ReleaseEntry {
        profile: "mock".into(),
        version: version.into(),
        baseline: to.into(),
        from_baseline: from.into(),
        file: format!("enkrnett-{version}-{to}-mock-sysupgrade.bin"),
        size: 6_000_000,
        sha512: "ab".repeat(64),
        min_free_kib: 24_576,
        notes_url: format!("https://example.org/releases/{version}"),
        install_seconds: 180,
    }
}

fn index(sequence: u64, releases: Vec<ReleaseEntry>) -> Vec<u8> {
    serde_json::to_vec(&ReleaseIndex {
        format: 1,
        sequence,
        releases,
    })
    .unwrap()
}

/// Serves `index` signed by `signer` at the mock's index URL.
fn serve(p: &MockPlatform, signer: &SecretKey, index: &[u8]) {
    p.set_fetch(URL, Ok(index.to_vec()));
    p.set_fetch(
        &format!("{URL}.sig"),
        Ok(signer
            .sign(index)
            .to_text("verify with release.pub")
            .into_bytes()),
    );
}

fn with_keys(p: &MockPlatform, keys: &[&SecretKey]) {
    p.set_release_keys(
        keys.iter()
            .map(|k| k.public_key().to_text("enkrnett release key"))
            .collect(),
    );
}

fn claim(ts: &TestServer) -> String {
    let (token, _) = common::claim(ts);
    token
}

fn check(ts: &TestServer, token: &str) -> serde_json::Value {
    let r = post(ts, "update/check", Some(token), "{}");
    assert_eq!(r.status, 200, "{}", r.body);
    r.json()
}

#[test]
fn a_newer_release_for_this_profile_and_baseline_is_offered() {
    let a = key(1);
    let ts = start_with(|p| {
        with_keys(p, &[&a]);
        serve(
            p,
            &a,
            &index(
                7,
                vec![
                    entry("0.3.0", "25.12", "25.12"),
                    entry("0.3.1", "25.12", "25.12"),
                    entry("0.9.0", "26.06", "26.06"),
                ],
            ),
        );
    });
    let token = claim(&ts);
    let u = check(&ts, &token);
    assert_eq!(u["last_check"]["result"], "ok");
    assert_eq!(u["available"]["version"], "0.3.1");
    assert_eq!(u["available"]["baseline"], "25.12");
    assert_eq!(u["available"]["changes_baseline"], false);
    assert_eq!(u["available"]["size_kib"], 5860);
    assert_eq!(u["available"]["install_seconds"], 180);
    assert_eq!(u["installed"]["version"], "0.2.0");
    // The status screen carries the same shape; GET update agrees.
    let s = get(&ts, "status", Some(&token)).json();
    assert_eq!(s["update"]["available"]["version"], "0.3.1");
    assert_eq!(
        get(&ts, "update", Some(&token)).json()["available"]["version"],
        "0.3.1"
    );
    // The index and its signature were fetched, nothing else (ENK-UPD-001: the same request
    // for every device, no identifier).
    let fetches: Vec<String> = ts
        .platform
        .calls()
        .into_iter()
        .filter(|c| c.starts_with("fetch("))
        .collect();
    assert_eq!(
        fetches,
        vec![format!("fetch({URL})"), format!("fetch({URL}.sig)")]
    );
}

#[test]
fn the_installed_or_an_older_release_is_not_offered() {
    let a = key(1);
    let ts = start_with(|p| {
        with_keys(p, &[&a]);
        serve(
            p,
            &a,
            &index(
                7,
                vec![
                    entry("0.2.0", "25.12", "25.12"),
                    entry("0.1.0", "25.12", "25.12"),
                ],
            ),
        );
    });
    let token = claim(&ts);
    let u = check(&ts, &token);
    assert_eq!(u["last_check"]["result"], "ok");
    assert_eq!(u["available"], serde_json::Value::Null);
    let r = post(&ts, "update/install", Some(&token), "{}");
    assert_eq!((r.status, r.code()), (409, "nothing_to_install".into()));
}

#[test]
fn an_upgrade_is_offered_from_the_installed_baseline_and_says_it_changes_it() {
    let a = key(1);
    let ts = start_with(|p| {
        with_keys(p, &[&a]);
        serve(p, &a, &index(7, vec![entry("0.4.0", "25.12", "26.06")]));
    });
    let token = claim(&ts);
    let u = check(&ts, &token);
    assert_eq!(u["available"]["version"], "0.4.0");
    assert_eq!(u["available"]["changes_baseline"], true);
    assert_eq!(u["available"]["baseline"], "26.06");
}

#[test]
fn an_index_with_a_lower_sequence_is_refused_and_the_sequence_survives_a_restart() {
    let a = key(1);
    let ts = start_with(|p| {
        with_keys(p, &[&a]);
        serve(p, &a, &index(7, vec![entry("0.3.0", "25.12", "25.12")]));
    });
    let token = claim(&ts);
    assert_eq!(check(&ts, &token)["available"]["version"], "0.3.0");
    // A replayed older index (threat 43): refused, and the offer already made stays.
    serve(
        &ts.platform,
        &a,
        &index(3, vec![entry("0.5.0", "25.12", "25.12")]),
    );
    let u = check(&ts, &token);
    assert_eq!(u["last_check"]["result"], "stale_index");
    assert_eq!(u["available"]["version"], "0.3.0");
    // The same sequence again is not stale.
    serve(
        &ts.platform,
        &a,
        &index(7, vec![entry("0.3.0", "25.12", "25.12")]),
    );
    assert_eq!(check(&ts, &token)["last_check"]["result"], "ok");
    // After a restart the device still knows sequence 7.
    let dir = ts.state_dir();
    let _first = ts; // alive, so that its directory stays
    let ts = start_on_with(&dir, |p| {
        with_keys(p, &[&a]);
        serve(p, &a, &index(6, vec![entry("0.3.0", "25.12", "25.12")]));
    });
    let token = login(&ts);
    let u = check(&ts, &token);
    assert_eq!(u["last_check"]["result"], "stale_index");
    assert_eq!(u["available"], serde_json::Value::Null);
}

#[test]
fn signatures_that_do_not_verify_and_indexes_that_do_not_parse_are_refused() {
    let a = key(1);
    let stranger = key(9);
    let good = index(7, vec![entry("0.3.0", "25.12", "25.12")]);
    let ts = start_with(|p| with_keys(p, &[&a]));
    let token = claim(&ts);
    let p = &ts.platform;
    // Signed by a key the image does not carry.
    serve(p, &stranger, &good);
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_signature");
    // Signed by the right key, over other bytes.
    serve(p, &a, &good);
    let mut tampered = good.clone();
    tampered[20] ^= 1;
    p.set_fetch(URL, Ok(tampered));
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_signature");
    // A signature file that is not one.
    p.set_fetch(URL, Ok(good.clone()));
    p.set_fetch(&format!("{URL}.sig"), Ok(b"not a signature".to_vec()));
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_signature");
    // Validly signed, but not an index.
    serve(p, &a, b"{\"format\": 1}");
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_index");
    serve(p, &a, &index(7, vec![entry("0.3", "25.12", "25.12")]));
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_index");
    // Too big to be the index.
    p.set_fetch(URL, Err(FetchError::TooLarge));
    assert_eq!(check(&ts, &token)["last_check"]["result"], "bad_index");
    // Nothing fetched at all, or no signature file beside the index.
    p.set_fetch(URL, Err(FetchError::Unreachable("timeout".into())));
    assert_eq!(check(&ts, &token)["last_check"]["result"], "unreachable");
    p.set_fetch(URL, Ok(good));
    p.set_fetch(
        &format!("{URL}.sig"),
        Err(FetchError::Unreachable("404".into())),
    );
    assert_eq!(check(&ts, &token)["last_check"]["result"], "unreachable");
    // Through all of this nothing was offered.
    assert_eq!(
        get(&ts, "update", Some(&token)).json()["available"],
        serde_json::Value::Null
    );
}

#[test]
fn the_designated_next_key_verifies_too_and_an_unparsable_key_file_is_ignored() {
    // ENK-RLS-005: the image carries the current key and one designated next key.
    let current = key(1);
    let next = key(2);
    let ts = start_with(|p| {
        p.set_release_keys(vec![
            current.public_key().to_text("current"),
            "garbage\n".into(),
            next.public_key().to_text("next"),
        ]);
        serve(p, &next, &index(8, vec![entry("0.3.0", "25.12", "25.12")]));
    });
    let token = claim(&ts);
    let u = check(&ts, &token);
    assert_eq!(u["last_check"]["result"], "ok");
    assert_eq!(u["available"]["version"], "0.3.0");
}

#[test]
fn a_package_install_does_not_look_for_updates_until_the_owner_turns_checks_on() {
    // ENK-UPD-008.
    let a = key(1);
    let ts = start_with(|p| {
        p.set_installed_as(enkrnett_model::api::InstalledAs::Package);
        with_keys(p, &[&a]);
        serve(p, &a, &index(7, vec![entry("0.3.0", "25.12", "25.12")]));
    });
    let token = claim(&ts);
    let u = get(&ts, "update", Some(&token)).json();
    assert_eq!(u["checks_enabled"], false);
    let u = check(&ts, &token);
    assert_eq!(u["checks_enabled"], false);
    assert_eq!(u["last_check"], serde_json::Value::Null);
    assert!(ts.platform.calls().iter().all(|c| !c.starts_with("fetch(")));
    // The switch, then the check in the same request.
    let r = post(
        &ts,
        "update/check",
        Some(&token),
        r#"{"enable_checks": true}"#,
    );
    assert_eq!(r.status, 200);
    let u = r.json();
    assert_eq!(u["checks_enabled"], true);
    assert_eq!(u["last_check"]["result"], "ok");
    assert_eq!(u["available"]["version"], "0.3.0");
    // The switch survives a restart.
    let dir = ts.state_dir();
    let _first = ts; // alive, so that its directory stays
    let ts = start_on_with(&dir, |p| {
        p.set_installed_as(enkrnett_model::api::InstalledAs::Package)
    });
    let token = login(&ts);
    assert_eq!(
        get(&ts, "update", Some(&token)).json()["checks_enabled"],
        true
    );
    // And can be turned off again; an offer is then withheld.
    let u = post(
        &ts,
        "update/check",
        Some(&token),
        r#"{"enable_checks": false}"#,
    )
    .json();
    assert_eq!(u["checks_enabled"], false);
    assert_eq!(u["available"], serde_json::Value::Null);
    // The body is strict.
    let r = post(&ts, "update/check", Some(&token), r#"{"enable": true}"#);
    assert_eq!((r.status, r.code()), (400, "invalid".into()));
}

#[test]
fn a_development_build_does_not_look_for_updates_unless_told_to() {
    let a = key(1);
    let ts = start_with_config(
        |c| c.development = true,
        |p| {
            with_keys(p, &[&a]);
            serve(p, &a, &index(7, vec![entry("0.3.0", "25.12", "25.12")]));
        },
    );
    let token = claim(&ts);
    assert_eq!(check(&ts, &token)["checks_enabled"], false);
    let u = post(
        &ts,
        "update/check",
        Some(&token),
        r#"{"enable_checks": true}"#,
    )
    .json();
    assert_eq!(u["available"]["version"], "0.3.0");
}

#[test]
fn without_an_index_url_the_check_reports_unreachable() {
    let ts = start_with(|p| p.set_update_index_url(None));
    let token = claim(&ts);
    assert_eq!(check(&ts, &token)["last_check"]["result"], "unreachable");
}
