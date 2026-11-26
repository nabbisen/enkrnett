//! The effective state and drift through the API, against the mock platform: ENK-API-027,
//! ENK-CFG-013, ENK-CFG-014, ENK-TOPO-003, ENK-CFG-009 (a restore cut short by a restart).

mod common;

use std::time::Duration;

use common::*;
use enkrnettd::platform::Platform;

fn effective_calls(ts: &TestServer) -> usize {
    ts.platform
        .calls()
        .iter()
        .filter(|c| *c == "effective()")
        .count()
}

#[test]
fn status_answers_from_readings_at_most_two_seconds_old() {
    // ENK-API-027: two clients polling cost one platform reading per 2 s.
    let ts = start();
    let (token, _) = claim(&ts);
    let before = effective_calls(&ts);
    let s1 = get(&ts, "status", Some(&token)).json();
    let s2 = get(&ts, "status", Some(&token)).json();
    assert_eq!(effective_calls(&ts), before + 1);
    assert_eq!(s1["guest"]["checked_ago"], 0);
    assert_eq!(s2["guest"]["checked_ago"], 0);
    ts.clock.advance(Duration::from_secs(1));
    let s3 = get(&ts, "status", Some(&token)).json();
    assert_eq!(effective_calls(&ts), before + 1, "still within 2 s");
    assert_eq!(s3["guest"]["checked_ago"], 1);
    ts.clock.advance(Duration::from_secs(1));
    get(&ts, "status", Some(&token));
    assert_eq!(effective_calls(&ts), before + 2, "read again after 2 s");
    // ENK-CFG-013: the effective state stands beside the desired document.
    let s = get(&ts, "status", Some(&token)).json();
    assert_eq!(s["guest"]["up"], true);
    assert_eq!(s["guest"]["name"], s["settings"]["guest"]["name"]);
    assert_eq!(s["guest"]["mode"], "open_to_all");
    assert_eq!(s["management"]["up"], true);
    assert_eq!(s["management"]["address"], "192.168.207.1");
}

#[test]
fn drift_is_reported_and_restored_only_on_request() {
    // ENK-CFG-014: a hand-edited section is reported, never overwritten by itself; restore is an
    // ordinary change.
    let ts = start();
    let (token, _) = claim(&ts);
    assert_eq!(get(&ts, "status", Some(&token)).json()["drift"], false);
    ts.platform.set_drift(true);
    // Nothing happens until a check is due; then the status says so.
    ts.app.check_drift_if_due(Duration::ZERO);
    let applies_before = ts
        .platform
        .calls()
        .iter()
        .filter(|c| c.starts_with("apply_settings"))
        .count();
    assert_eq!(get(&ts, "status", Some(&token)).json()["drift"], true);
    assert_eq!(
        ts.platform
            .calls()
            .iter()
            .filter(|c| c.starts_with("apply_settings"))
            .count(),
        applies_before,
        "the daemon does not overwrite drift by itself"
    );
    let r = post(&ts, "settings/restore", Some(&token), "{}");
    assert_eq!(r.status, 202, "{}", r.body);
    assert_eq!(r.json()["kind"], "restore");
    wait_for_change_state(&ts, &token, "settled");
    assert_eq!(get(&ts, "status", Some(&token)).json()["drift"], false);
}

#[test]
fn the_status_itself_checks_for_drift_but_not_more_than_every_thirty_seconds() {
    let ts = start();
    let (token, _) = claim(&ts);
    get(&ts, "status", Some(&token));
    ts.platform.set_drift(true);
    assert_eq!(
        get(&ts, "status", Some(&token)).json()["drift"],
        false,
        "checked less than 30 s ago"
    );
    ts.clock.advance(Duration::from_secs(30));
    assert_eq!(get(&ts, "status", Some(&token)).json()["drift"], true);
}

#[test]
fn at_start_up_the_device_is_adopted_and_drift_is_not_overwritten() {
    // ENK-CFG-014 at start-up: compare, report, leave alone.
    let ts = start();
    let (_token, _) = claim(&ts);
    let dir = ts.state_dir();
    let second = start_on_with(&dir, |p| p.set_drift(true));
    let calls = second.platform.calls();
    assert!(
        calls.iter().any(|c| c.starts_with("adopt_settings(v1")),
        "{calls:?}"
    );
    assert!(
        !calls.iter().any(|c| c.starts_with("apply_settings")),
        "{calls:?}"
    );
    let login = post(
        &second,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    let token = login.json()["token"].as_str().unwrap().to_string();
    assert_eq!(get(&second, "status", Some(&token)).json()["drift"], true);
}

#[test]
fn a_restore_cut_short_by_a_restart_is_completed_by_the_start_up() {
    use enkrnett_model::STORAGE_FORMAT;
    use enkrnett_model::api::ChangeKind;
    use enkrnettd::store::{PendingMarker, Store};
    let ts = start();
    let (_token, _) = claim(&ts);
    let dir = ts.state_dir();
    let store = Store::open(&dir, || "x".repeat(16)).unwrap();
    let previous = store.data.owner.clone().unwrap();
    store
        .write_pending(&PendingMarker {
            format: STORAGE_FORMAT,
            id: 4,
            kind: ChangeKind::Restore,
            from_version: 1,
            to_version: 1,
            needs_confirmation: false,
            previous,
        })
        .unwrap();
    let second = start_on(&dir);
    let calls = second.platform.calls();
    assert!(
        calls.iter().any(|c| c.starts_with("apply_settings(v1")),
        "the start-up apply is the restore: {calls:?}"
    );
    let login = post(
        &second,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    let token = login.json()["token"].as_str().unwrap().to_string();
    let c = get(&second, "change", Some(&token)).json();
    assert_eq!(c["id"], 4);
    assert_eq!(c["state"], "settled");
    assert_eq!(c["kind"], "restore");
    assert!(!dir.join("pending.json").exists());
}

#[test]
fn an_uplink_collision_renumbers_without_an_owner_visible_change() {
    // ENK-TOPO-003: noticed by the periodic check; the platform re-applies and renumbers.
    let ts = start();
    let (token, _) = claim(&ts);
    let change_before = get(&ts, "change", Some(&token)).json()["id"].clone();
    ts.platform.set_uplink_collides(true);
    let applies_before = ts
        .platform
        .calls()
        .iter()
        .filter(|c| c.starts_with("apply_settings"))
        .count();
    ts.app.check_uplink_if_due(Duration::ZERO);
    assert_eq!(
        ts.platform
            .calls()
            .iter()
            .filter(|c| c.starts_with("apply_settings"))
            .count(),
        applies_before + 1
    );
    assert!(!ts.platform.uplink_collides());
    assert_eq!(
        get(&ts, "change", Some(&token)).json()["id"],
        change_before,
        "no change record"
    );
    // At start-up with a collision, adopting writes.
    let dir = ts.state_dir();
    let second = start_on_with(&dir, |p| p.set_uplink_collides(true));
    let calls = second.platform.calls();
    assert!(calls.iter().any(|c| c.starts_with("adopt_settings")));
    assert!(
        calls.iter().any(|c| c.starts_with("apply_settings")),
        "{calls:?}"
    );
}
