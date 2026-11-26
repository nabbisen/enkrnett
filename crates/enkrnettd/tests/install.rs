//! Installing an update against the mock platform (chapter 11 §3): ENK-UPD-002 to 007,
//! ENK-UPD-004 and 005 across a restart as the successor release, ENK-API-012.

mod common;

use std::time::Duration;

use common::*;
use enkrnett_model::api::{HealthDetail, UpdateFailure};
use enkrnett_model::release::{ReleaseEntry, ReleaseIndex};
use enkrnett_model::signify::{SecretKey, hex, key_pair, sha512};
use enkrnettd::platform::{InstallVerdict, MockPlatform};
use serde_json::Value;

const URL: &str = "https://updates.example/enkrnett/index.json";
const IMAGE: &str = "https://updates.example/enkrnett/enkrnett-0.3.0-25.12-mock-sysupgrade.bin";

fn key(n: u8) -> SecretKey {
    key_pair([n; 32], [n; 8], [0; 16])
}

/// A pretend image: 300 KB that are not all alike.
fn image_bytes() -> Vec<u8> {
    (0..300_000u32).map(|i| (i * 7 % 253) as u8).collect()
}

fn entry(image: &[u8]) -> ReleaseEntry {
    ReleaseEntry {
        profile: "mock".into(),
        version: "0.3.0".into(),
        baseline: "25.12".into(),
        from_baseline: "25.12".into(),
        file: "enkrnett-0.3.0-25.12-mock-sysupgrade.bin".into(),
        size: image.len() as u64,
        sha512: hex(&sha512(image)),
        min_free_kib: 16_384,
        notes_url: "https://example.org/releases/0.3.0".into(),
        install_seconds: 180,
    }
}

fn index(entry: ReleaseEntry) -> Vec<u8> {
    serde_json::to_vec(&ReleaseIndex {
        format: 1,
        sequence: 7,
        releases: vec![entry],
    })
    .unwrap()
}

/// The index and the image, signed by `signer`, with the key `holder` installed on the "device".
fn serve(p: &MockPlatform, holder: &SecretKey, signer: &SecretKey, index: &[u8], image: &[u8]) {
    p.set_update_index_url(Some(URL));
    p.set_release_keys(vec![holder.public_key().to_text("enkrnett release key")]);
    p.set_fetch(URL, Ok(index.to_vec()));
    p.set_fetch(
        &format!("{URL}.sig"),
        Ok(holder
            .sign(index)
            .to_text("verify with release.pub")
            .into_bytes()),
    );
    p.set_fetch(IMAGE, Ok(image.to_vec()));
    p.set_fetch(
        &format!("{IMAGE}.sig"),
        Ok(signer
            .sign(image)
            .to_text("verify with release.pub")
            .into_bytes()),
    );
}

fn serve_good(p: &MockPlatform, k: &SecretKey) {
    let image = image_bytes();
    serve(p, k, k, &index(entry(&image)), &image);
}

fn claim_and_check(ts: &TestServer) -> String {
    let (token, _) = claim(ts);
    let r = post(ts, "update/check", Some(&token), "{}");
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["available"]["version"], "0.3.0", "{}", r.body);
    token
}

fn update(ts: &TestServer, token: &str) -> Value {
    get(ts, "update", Some(token)).json()
}

/// Waits for `update.state` to become `state`.
fn wait_for_update_state(ts: &TestServer, token: &str, state: &str) -> Value {
    for _ in 0..600 {
        let u = update(ts, token);
        if u["state"] == state {
            return u;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "update did not reach {state}: {}",
        get(ts, "update", Some(token)).body
    );
}

fn install(ts: &TestServer, token: &str) -> Resp {
    post(ts, "update/install", Some(token), "{}")
}

fn marker(ts: &TestServer) -> Value {
    serde_json::from_slice(&std::fs::read(ts.state_dir().join("update.json")).unwrap()).unwrap()
}

#[test]
fn the_owner_installs_the_offered_release_and_the_successor_reports_it_done() {
    let k = key(1);
    let ts = start_with(|p| serve_good(p, &k));
    let token = claim_and_check(&ts);
    assert_eq!(update(&ts, &token)["can_undo"], false, "ENK-UPD-006");

    // ENK-UPD-002: only the owner's request starts it; 202 with the shape.
    let r = install(&ts, &token);
    assert_eq!(r.status, 202, "{}", r.body);
    assert!(
        ["downloading", "verifying", "installing"].contains(&r.json()["state"].as_str().unwrap())
    );
    let u = wait_for_update_state(&ts, &token, "installing");
    assert_eq!(u["available"]["version"], "0.3.0");
    assert_eq!(u["last_result"], Value::Null);

    // ENK-API-012: every POST but logout is refused meanwhile; reads go on.
    let r = post(&ts, "update/check", Some(&token), "{}");
    assert_eq!(r.status, 409, "{}", r.body);
    assert_eq!(r.json()["error"]["code"], "update_in_progress");
    assert_eq!(post(&ts, "restart", Some(&token), "{}").status, 409);
    assert_eq!(post(&ts, "management-key", Some(&token), "{}").status, 409);
    assert_eq!(get(&ts, "status", Some(&token)).status, 200);

    // What the platform was asked: the image downloaded beside the index, its signature
    // fetched, then the staged files handed over (and nothing else of the device touched).
    let calls = ts.platform.calls();
    assert!(calls.contains(&format!("download({IMAGE})")), "{calls:?}");
    assert!(calls.contains(&format!("fetch({IMAGE}.sig)")), "{calls:?}");
    assert!(
        calls.contains(&"install(image, image.sig)".to_string()),
        "{calls:?}"
    );
    assert!(!calls.iter().any(|c| c.starts_with("apply_settings")
        && calls.iter().position(|x| x == c)
            > calls.iter().position(|x| x.starts_with("install("))));
    let staging = ts.state_dir().join("staging");
    assert_eq!(std::fs::read(staging.join("image")).unwrap(), image_bytes());
    assert!(staging.join("image.sig").exists());
    // The marker is on flash before the hand-over (ENK-UPD-005).
    let m = marker(&ts);
    assert_eq!(m["stage"], "installing");
    assert_eq!(m["from_version"], "0.2.0");
    assert_eq!(m["to_version"], "0.3.0");
    assert_eq!(m["had_owner"], true);
    // Logging out is the one POST still allowed.
    assert_eq!(post(&ts, "logout", Some(&token), "{}").status, 200);

    // The first boot of the successor (ENK-UPD-005): the same directory, version 0.3.0, a
    // healthy start. The owner, the key and the settings came along (ENK-UPD-004); the old
    // session did not.
    let dir = ts.state_dir();
    let _first = ts; // alive, so that its directory stays
    let ts = start_on_configured(&dir, |c| c.version = "0.3.0".into(), |p| serve_good(p, &k));
    assert_eq!(get(&ts, "update", Some(&token)).status, 401);
    let token = login(&ts);
    let u = update(&ts, &token);
    assert_eq!(u["installed"]["version"], "0.3.0");
    assert_eq!(u["state"], "idle");
    assert_eq!(u["last_result"]["result"], "done", "{u}");
    assert_eq!(u["last_result"]["reason"], Value::Null);
    let s = get(&ts, "status", Some(&token)).json();
    assert_eq!(s["settings"]["guest"]["name"], "Cafe Example");
    assert_eq!(s["version"]["enkrnett"], "0.3.0");
    assert_eq!(marker(&ts)["stage"], "done");
    assert!(
        !dir.join("staging").exists(),
        "the staged image is gone from RAM"
    );
    // Up to date now: nothing offered, the result stays on the details screen.
    let r = post(&ts, "update/check", Some(&token), "{}");
    assert_eq!(r.json()["available"], Value::Null);
    assert_eq!(r.json()["last_result"]["result"], "done");
    // A plain restart keeps the outcome (kept across the restart, chapter 9 §4).
    let _second = ts;
    let ts = start_on_configured(&dir, |c| c.version = "0.3.0".into(), |_| {});
    let token = login(&ts);
    assert_eq!(update(&ts, &token)["last_result"]["result"], "done");
}

#[test]
fn a_broken_successor_fails_its_health_check_and_the_device_keeps_serving_its_settings() {
    let k = key(1);
    let ts = start_with(|p| serve_good(p, &k));
    let token = claim_and_check(&ts);
    assert_eq!(install(&ts, &token).status, 202);
    wait_for_update_state(&ts, &token, "installing");
    let dir = ts.state_dir();
    let _first = ts;
    // ENK-UPD-005: the start-up health check of the new release fails.
    let ts = start_on_configured(
        &dir,
        |c| c.version = "0.3.0".into(),
        |p| p.fail_next_apply(HealthDetail::GuestDown),
    );
    let token = login(&ts);
    let u = update(&ts, &token);
    assert_eq!(u["installed"]["version"], "0.3.0");
    assert_eq!(u["last_result"]["result"], "failed", "{u}");
    assert_eq!(u["last_result"]["reason"], "health_check_failed");
    assert_eq!(u["state"], "idle");
    // The settings are what they were; the status screen says so (it carries `update`).
    let s = get(&ts, "status", Some(&token)).json();
    assert_eq!(s["settings"]["guest"]["name"], "Cafe Example");
    assert_eq!(s["update"]["last_result"]["reason"], "health_check_failed");
    assert_eq!(marker(&ts)["reason"], "health_check_failed");
    // Changes are possible again: the device is not stuck in an update.
    assert_eq!(post(&ts, "management-key", Some(&token), "{}").status, 202);
}

#[test]
fn the_root_side_may_refuse_or_fail_and_an_attempt_that_did_not_take_is_reported() {
    let k = key(1);
    let ts = start_with(|p| {
        serve_good(p, &k);
        p.set_install_verdict(None);
    });
    let token = claim_and_check(&ts);
    assert_eq!(install(&ts, &token).status, 202);
    wait_for_update_state(&ts, &token, "installing");
    // No verdict yet: still installing; a second install is refused as in progress.
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(update(&ts, &token)["state"], "installing");
    assert_eq!(
        install(&ts, &token).json()["error"]["code"],
        "update_in_progress"
    );
    // Refused by the root side (the system said the image is not for this device).
    ts.platform
        .set_install_verdict(Some(InstallVerdict::Refused(UpdateFailure::WrongDevice)));
    let u = wait_for_update_state(&ts, &token, "idle");
    assert_eq!(u["last_result"]["result"], "failed");
    assert_eq!(u["last_result"]["reason"], "wrong_device");
    assert!(!ts.state_dir().join("staging").exists());
    assert_eq!(marker(&ts)["stage"], "failed");
    // The offer stands; the next attempt starts afresh and its failure replaces the outcome.
    ts.platform
        .set_install_verdict(Some(InstallVerdict::Failed));
    assert_eq!(install(&ts, &token).status, 202);
    let u = wait_for_update_state(&ts, &token, "idle");
    assert_eq!(u["last_result"]["reason"], "install_failed");
    // Flashing announced, then the "device" comes back as the same release: the first boot
    // finds an `installing` marker for a version that is not running.
    ts.platform
        .set_install_verdict(Some(InstallVerdict::Flashing));
    assert_eq!(install(&ts, &token).status, 202);
    wait_for_update_state(&ts, &token, "installing");
    assert_eq!(marker(&ts)["stage"], "installing");
    let dir = ts.state_dir();
    let _first = ts;
    let ts = start_on_with(&dir, |p| serve_good(p, &k));
    let token = login(&ts);
    let u = update(&ts, &token);
    assert_eq!(u["installed"]["version"], "0.2.0");
    assert_eq!(u["last_result"]["reason"], "install_failed", "{u}");
    assert_eq!(u["state"], "idle");
}

#[test]
fn the_checks_before_the_download_refuse_with_the_reason_and_write_nothing() {
    let k = key(1);
    // ENK-UPD-003: free memory below the image plus 16 MB.
    let ts = start_with(|p| {
        serve_good(p, &k);
        p.set_available_memory_kib(16_384);
    });
    let token = claim_and_check(&ts);
    let r = install(&ts, &token);
    assert_eq!(r.status, 400, "{}", r.body);
    assert_eq!(r.json()["error"]["code"], "cannot_update");
    assert_eq!(r.json()["error"]["reason"], "not_enough_memory");
    let u = update(&ts, &token);
    assert_eq!(u["state"], "idle");
    assert_eq!(
        u["last_result"],
        Value::Null,
        "a refusal before the attempt is not an attempt"
    );
    assert!(
        !ts.platform
            .calls()
            .iter()
            .any(|c| c.starts_with("download("))
    );
    assert!(!ts.state_dir().join("staging").exists());
    assert!(!ts.state_dir().join("update.json").exists());
    // Memory back: a change in progress refuses with the change (ENK-UPD-003, chapter 9 §5).
    ts.platform.set_available_memory_kib(98_304);
    ts.platform.hold_apply();
    let r = post(
        &ts,
        "settings",
        Some(&token),
        r#"{"version":1,"guest":{"name":"Cafe Example 2","mode":"open_to_all","enabled":true},"management":{"name":"Enkrnett-3F2A"}}"#,
    );
    assert_eq!(r.status, 202, "{}", r.body);
    let r = install(&ts, &token);
    assert_eq!(r.status, 409, "{}", r.body);
    assert_eq!(r.json()["error"]["code"], "change_pending");
    assert_eq!(r.json()["error"]["change"]["state"], "applying");
    ts.platform.release_apply();
    wait_for_change_state(&ts, &token, "settled");

    // Nothing offered: nothing to install, also when checks are off (ENK-UPD-008).
    let ts = start_with(|p| p.set_release_keys(Vec::new()));
    let (token, _) = claim(&ts);
    let r = install(&ts, &token);
    assert_eq!(r.status, 409);
    assert_eq!(r.json()["error"]["code"], "nothing_to_install");
    let ts = start_with_config(|c| c.development = true, |p| serve_good(p, &k));
    let (token, _) = claim(&ts);
    assert_eq!(
        post(&ts, "update/check", Some(&token), "{}").json()["available"],
        Value::Null
    );
    assert_eq!(
        install(&ts, &token).json()["error"]["code"],
        "nothing_to_install"
    );
}

#[test]
fn bytes_that_do_not_match_the_index_or_the_signature_are_refused_and_nothing_is_handed_over() {
    let k = key(1);
    let stranger = key(9);
    let image = image_bytes();
    let mut other = image.clone();
    other[1000] ^= 0xff;
    type Setup<'a> = Box<dyn Fn(&MockPlatform) + 'a>;
    let cases: Vec<(&str, Setup, &str)> = vec![
        (
            "a changed byte: the hash and the signature differ from the index",
            Box::new(|p| serve(p, &k, &k, &index(entry(&image)), &other)),
            "bad_signature",
        ),
        (
            "a stranger's signature over the right bytes",
            Box::new(|p| serve(p, &k, &stranger, &index(entry(&image)), &image)),
            "bad_signature",
        ),
        (
            "more bytes than the index says",
            Box::new(|p| {
                let mut e = entry(&image);
                e.size -= 1;
                serve(p, &k, &k, &index(e), &image)
            }),
            "bad_signature",
        ),
        (
            "the image cannot be downloaded",
            Box::new(|p| {
                serve(p, &k, &k, &index(entry(&image)), &image);
                p.set_fetch(
                    IMAGE,
                    Err(enkrnettd::platform::FetchError::Unreachable("gone".into())),
                );
            }),
            "download_failed",
        ),
        (
            "the signature file cannot be downloaded",
            Box::new(|p| {
                serve(p, &k, &k, &index(entry(&image)), &image);
                p.set_fetch(
                    &format!("{IMAGE}.sig"),
                    Err(enkrnettd::platform::FetchError::Unreachable("gone".into())),
                );
            }),
            "download_failed",
        ),
    ];
    for (what, setup, reason) in cases {
        let ts = start_with(|p| setup(p));
        let token = claim_and_check(&ts);
        assert_eq!(install(&ts, &token).status, 202, "{what}");
        let u = wait_for_update_state(&ts, &token, "idle");
        assert_eq!(u["last_result"]["result"], "failed", "{what}: {u}");
        assert_eq!(u["last_result"]["reason"], reason, "{what}: {u}");
        assert!(
            !ts.platform
                .calls()
                .iter()
                .any(|c| c.starts_with("install(")),
            "{what}: nothing was handed over"
        );
        assert!(
            !ts.state_dir().join("staging").exists(),
            "{what}: RAM freed"
        );
        assert_eq!(marker(&ts)["reason"], reason, "{what}");
        // The offer stands and so does the device: the owner may try again.
        assert_eq!(u["available"]["version"], "0.3.0", "{what}");
    }
}
