//! The transaction of chapter 6 §4 through the API, against the mock platform:
//! ENK-CFG-005, 006, 008, 009, 010, 011, 012; ENK-API-021, 022, 023.

mod common;

use std::time::Duration;

use common::*;
use enkrnett_model::api::HealthDetail;
use enkrnett_model::key::is_management_key;
use serde_json::Value;

fn save(
    ts: &TestServer,
    token: &str,
    version: u64,
    guest_name: &str,
    management_name: &str,
) -> Resp {
    post(
        ts,
        "settings",
        Some(token),
        &format!(
            r#"{{"version":{version},"guest":{{"name":"{guest_name}","mode":"open_to_all","enabled":true}},"management":{{"name":"{management_name}"}}}}"#
        ),
    )
}

fn change_state(ts: &TestServer, token: &str) -> Value {
    get(ts, "change", Some(token)).json()
}

#[test]
fn a_guest_change_is_accepted_as_applying_and_settles_by_itself() {
    // ENK-CFG-010: 202 while applying; settled once the health check passed.
    let ts = start();
    let (token, _) = claim(&ts);
    ts.platform.hold_apply();
    let r = save(&ts, &token, 1, "Cafe Example 2", "Enkrnett-3F2A");
    assert_eq!(r.status, 202, "{}", r.body);
    let c = r.json();
    assert_eq!(c["state"], "applying");
    assert_eq!(c["kind"], "settings");
    assert_eq!(c["from_version"], 1);
    assert_eq!(c["to_version"], 2);
    assert_eq!(c["needs_confirmation"], false);
    assert_eq!(c["confirm_in"], Value::Null);
    // ENK-CFG-006: the marker is on flash before the platform is asked.
    assert!(ts.state_dir().join("pending.json").exists());
    assert_eq!(
        get(&ts, "status", Some(&token)).json()["settings"]["version"],
        1
    );
    ts.platform.release_apply();
    let settled = wait_for_change_state(&ts, &token, "settled");
    assert_eq!(settled["id"], c["id"]);
    assert_eq!(settled["reason"], Value::Null);
    assert!(!ts.state_dir().join("pending.json").exists());
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(status["settings"]["version"], 2);
    assert_eq!(status["settings"]["guest"]["name"], "Cafe Example 2");
    assert_eq!(ts.platform.applied().unwrap().version, 2);
}

#[test]
fn one_change_at_a_time() {
    // ENK-CFG-011: the second is refused with the state of the first.
    let ts = start();
    let (token, _) = claim(&ts);
    ts.platform.hold_apply();
    let first = save(&ts, &token, 1, "Cafe Example 2", "Enkrnett-3F2A");
    assert_eq!(first.status, 202);
    let second = save(&ts, &token, 1, "Cafe Example 3", "Enkrnett-3F2A");
    assert_eq!(second.status, 409, "{}", second.body);
    let e = second.json();
    assert_eq!(e["error"]["code"], "change_pending");
    assert_eq!(e["error"]["change"]["state"], "applying");
    assert_eq!(e["error"]["change"]["id"], first.json()["id"]);
    let key = post(&ts, "management-key", Some(&token), "{}");
    assert_eq!(key.status, 409);
    let restore = post(&ts, "settings/restore", Some(&token), "{}");
    assert_eq!(restore.status, 409);
    ts.platform.release_apply();
    wait_for_change_state(&ts, &token, "settled");
    let third = save(&ts, &token, 2, "Cafe Example 3", "Enkrnett-3F2A");
    assert_eq!(third.status, 202, "{}", third.body);
}

#[test]
fn a_failed_health_check_undoes_the_change_and_says_why() {
    // ENK-CFG-009: the previous document is back on the device; the version did not move.
    let ts = start();
    let (token, _) = claim(&ts);
    ts.platform.fail_next_apply(HealthDetail::GuestDown);
    let r = save(&ts, &token, 1, "Cafe Example 2", "Enkrnett-3F2A");
    assert_eq!(r.status, 202);
    let undone = wait_for_change_state(&ts, &token, "undone");
    assert_eq!(undone["reason"], "health_check_failed");
    assert_eq!(undone["detail"], "guest_down");
    assert_eq!(undone["to_version"], 2);
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(status["settings"]["version"], 1);
    assert_eq!(status["settings"]["guest"]["name"], "Cafe Example");
    assert!(!ts.state_dir().join("pending.json").exists());
    let applied = ts.platform.applied().unwrap();
    assert_eq!(applied.version, 1);
    assert_eq!(applied.guest.name, "Cafe Example");
    let calls = ts.platform.calls();
    let applies: Vec<&String> = calls
        .iter()
        .filter(|c| c.starts_with("apply_settings"))
        .collect();
    assert!(
        applies
            .last()
            .unwrap()
            .starts_with("apply_settings(v1, Cafe Example,"),
        "{calls:?}"
    );
    // The next change is accepted: nothing is stuck.
    assert_eq!(
        save(&ts, &token, 1, "Cafe Example 2", "Enkrnett-3F2A").status,
        202
    );
}

#[test]
fn a_management_change_waits_for_a_request_on_a_new_connection() {
    // ENK-CFG-008, ENK-API-021, 022: disconnect, then the first owner request on a connection
    // accepted after the disconnection confirms. A request accepted before does not.
    let ts = start();
    let (token, _) = claim(&ts);
    ts.platform.hold_apply();
    let r = save(&ts, &token, 1, "Cafe Example", "Office");
    assert_eq!(r.status, 202, "{}", r.body);
    assert_eq!(r.json()["needs_confirmation"], true);
    // Requests while applying (before any disconnection) change nothing.
    assert_eq!(get(&ts, "status", Some(&token)).status, 200);
    assert_eq!(change_state(&ts, &token)["state"], "applying");
    ts.platform.release_apply();
    let waiting = wait_for_change_state(&ts, &token, "waiting_confirmation");
    assert_eq!(waiting["confirm_in"], 600);
    let calls = ts.platform.calls();
    let apply = calls
        .iter()
        .rposition(|c| c.starts_with("apply_settings(v2"))
        .unwrap();
    let disconnect = calls
        .iter()
        .rposition(|c| c == "disconnect_management_stations()")
        .expect("stations disconnected");
    assert!(disconnect > apply, "{calls:?}");
    // Same clock instant as the disconnection: not "after", so not a confirmation.
    assert_eq!(change_state(&ts, &token)["state"], "waiting_confirmation");
    assert_eq!(
        get(&ts, "status", Some(&token)).json()["settings"]["version"],
        1
    );
    // A later request without the owner's session does not confirm either.
    ts.clock.advance(Duration::from_secs(5));
    let stranger = get(&ts, "change", Some("not-a-session"));
    assert_eq!(stranger.status, 401);
    let inside = ts.app.change_get(Some(&token)).unwrap().body;
    assert_eq!(inside["state"], "waiting_confirmation");
    assert_eq!(inside["confirm_in"], 595);
    // The owner's first request on a connection accepted after the disconnection: this very
    // response already says settled.
    ts.clock.advance(Duration::from_secs(1));
    let confirming = get(&ts, "status", Some(&token)).json();
    assert_eq!(confirming["change"]["state"], "settled");
    assert_eq!(confirming["settings"]["version"], 2);
    assert_eq!(confirming["settings"]["management"]["name"], "Office");
    assert_eq!(ts.platform.management().unwrap().0, "Office");
    assert!(!ts.state_dir().join("pending.json").exists());
}

#[test]
fn a_management_change_not_confirmed_in_time_is_undone() {
    // ENK-CFG-008, ENK-CFG-012: 600 s on the monotonic clock, then the old network is back.
    let ts = start();
    let (token, _) = claim(&ts);
    let r = save(&ts, &token, 1, "Cafe Example", "Office");
    assert_eq!(r.status, 202);
    wait_for_change_state(&ts, &token, "waiting_confirmation");
    // From here on the owner sends nothing: any request of theirs on a later connection would
    // be the confirmation. The state is read inside the daemon instead.
    let observe = |ts: &TestServer| ts.app.change_get(Some(&token)).unwrap().body;
    ts.clock.advance(Duration::from_secs(599));
    ts.app.tick();
    assert_eq!(observe(&ts)["state"], "waiting_confirmation");
    assert_eq!(observe(&ts)["confirm_in"], 1);
    ts.clock.advance(Duration::from_secs(1));
    // The tick notices the deadline.
    ts.app.tick();
    for _ in 0..200 {
        if observe(&ts)["state"] == "undone" {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let undone = observe(&ts);
    assert_eq!(undone["state"], "undone");
    assert_eq!(undone["reason"], "not_confirmed");
    assert_eq!(undone["confirm_in"], Value::Null);
    assert_eq!(ts.platform.management().unwrap().0, "Enkrnett-3F2A");
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(status["settings"]["version"], 1);
    assert_eq!(status["settings"]["management"]["name"], "Enkrnett-3F2A");
}

#[test]
fn a_late_request_after_the_deadline_does_not_confirm() {
    let ts = start();
    let (token, _) = claim(&ts);
    save(&ts, &token, 1, "Cafe Example", "Office");
    wait_for_change_state(&ts, &token, "waiting_confirmation");
    ts.clock.advance(Duration::from_secs(600));
    // The request itself runs the tick first, so the deadline wins over the request.
    let late = get(&ts, "status", Some(&token)).json();
    assert_ne!(late["change"]["state"], "settled");
    assert_eq!(late["settings"]["management"]["name"], "Enkrnett-3F2A");
    let undone = wait_for_change_state(&ts, &token, "undone");
    assert_eq!(undone["reason"], "not_confirmed");
}

#[test]
fn replacing_the_management_key_is_a_confirmed_change_with_the_key_beside_it() {
    // ENK-API-023: the only compound response.
    let ts = start();
    let (token, _) = claim(&ts);
    let old_key = get(&ts, "management-key", Some(&token)).json()["key"]
        .as_str()
        .unwrap()
        .to_string();
    let r = post(&ts, "management-key", Some(&token), "{}");
    assert_eq!(r.status, 202, "{}", r.body);
    let body = r.json();
    assert_eq!(body["kind"], "management_key");
    assert_eq!(body["needs_confirmation"], true);
    let new_key = body["management_key"]["key"].as_str().unwrap().to_string();
    assert_ne!(new_key, old_key);
    assert!(is_management_key(&new_key));
    assert_eq!(body["management_key"]["name"], "Enkrnett-3F2A");
    wait_for_change_state(&ts, &token, "waiting_confirmation");
    assert_eq!(
        get(&ts, "management-key", Some(&token)).json()["key"],
        old_key
    );
    ts.clock.advance(Duration::from_secs(1));
    assert_eq!(get(&ts, "change", Some(&token)).json()["state"], "settled");
    assert_eq!(
        get(&ts, "management-key", Some(&token)).json()["key"],
        new_key
    );
    assert_eq!(ts.platform.management().unwrap().1, new_key);
}

#[test]
fn restore_rewrites_the_device_without_moving_the_version() {
    let ts = start();
    let (token, _) = claim(&ts);
    let r = post(&ts, "settings/restore", Some(&token), "{}");
    assert_eq!(r.status, 202, "{}", r.body);
    let c = r.json();
    assert_eq!(c["kind"], "restore");
    assert_eq!(c["from_version"], 1);
    assert_eq!(c["to_version"], 1);
    assert_eq!(c["needs_confirmation"], false);
    wait_for_change_state(&ts, &token, "settled");
    assert_eq!(
        get(&ts, "status", Some(&token)).json()["settings"]["version"],
        1
    );
}

#[test]
fn a_marker_left_by_a_restart_means_the_change_is_undone() {
    // ENK-CFG-009: start with a pending marker on flash.
    use enkrnett_model::STORAGE_FORMAT;
    use enkrnett_model::api::ChangeKind;
    use enkrnettd::store::{OwnerRecord, PendingMarker, Store};
    let ts = start();
    let (token, _) = claim(&ts);
    let dir = ts.state_dir();
    // The owner document of the first daemon, plus a marker as if a change had been cut short.
    let store = Store::open(&dir, || "x".repeat(16)).unwrap();
    let previous: OwnerRecord = store.data.owner.clone().unwrap();
    store
        .write_pending(&PendingMarker {
            format: STORAGE_FORMAT,
            id: 7,
            kind: ChangeKind::Settings,
            from_version: 1,
            to_version: 2,
            needs_confirmation: true,
            previous: previous.clone(),
        })
        .unwrap();
    drop(token);
    let second = start_on(&dir);
    assert!(!dir.join("pending.json").exists());
    let login = post(
        &second,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!(login.status, 200, "{}", login.body);
    let token = login.json()["token"].as_str().unwrap().to_string();
    let c = change_state(&second, &token);
    assert_eq!(c["id"], 7);
    assert_eq!(c["state"], "undone");
    assert_eq!(c["reason"], "restart");
    assert_eq!(c["kind"], "settings");
    assert_eq!(c["to_version"], 2);
    // The previous document was applied at start-up, as at every start.
    assert_eq!(second.platform.applied().unwrap().version, 1);
    assert_eq!(
        second.platform.management().unwrap().0,
        previous.management_name
    );
    // The next change takes the next number.
    let r = save(&second, &token, 1, "Cafe Example 2", "Enkrnett-3F2A");
    assert_eq!(r.json()["id"], 8);
}
