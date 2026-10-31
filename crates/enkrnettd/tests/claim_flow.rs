//! Chapter 4 over the API: the claim from `SETUP` to `CLAIMED`, the owner's operations, Erase.
//! ENK-OWN-004 to 020, ENK-AUTH-001 to 005, ENK-API-012, 014, 015, 017, 020, 023; ENK-CFG-016.

mod common;

use std::time::Duration;

use common::*;
use enkrnett_model::key::is_management_key;
use enkrnett_model::state::ConnectionSecurity;
use serde_json::json;

#[test]
fn only_one_claim_session_and_claim_describes_setup() {
    let ts = start();
    let first = post(&ts, "claim/start", None, "{}");
    assert_eq!(first.status, 200);
    assert_eq!(first.json()["kind"], "claim");
    assert_eq!(first.json()["idle_ends_in"], 300);
    let second = post(&ts, "claim/start", None, "{}");
    assert_eq!(
        (second.status, second.code()),
        (409, "setup_in_progress_elsewhere".into())
    );
    let token = first.json()["token"].as_str().unwrap().to_string();
    assert_eq!(token.len(), 43, "32 random bytes as base64url");
    let claim = get(&ts, "claim", Some(&token));
    assert_eq!(claim.status, 200);
    let v = claim.json();
    assert!(v["window_ends_in"].as_u64().unwrap() <= 1800);
    assert!(v["window_ends_in"].as_u64().unwrap() > 1700);
    assert_eq!(v["country"]["fixed"], serde_json::Value::Null);
    assert!(
        v["country"]["choices"]
            .as_array()
            .unwrap()
            .contains(&json!("JP"))
    );
    assert_eq!(v["guest_mode_default"], "open_to_all");
    assert_eq!(v["identify_available"], true);
    assert_eq!(v["limits"]["guest_name_max_bytes"]["open_to_all"], 29);
    assert_eq!(v["connection"]["security"], "owe");
    assert_eq!(post(&ts, "claim/identify", Some(&token), "{}").status, 200);
    assert_eq!(ts.platform.calls().last().unwrap(), "blink()");
}

#[test]
fn identify_is_not_available_without_a_light() {
    let ts = start_with(|p| p.set_light(false));
    let token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        get(&ts, "claim", Some(&token)).json()["identify_available"],
        false
    );
    let r = post(&ts, "claim/identify", Some(&token), "{}");
    assert_eq!((r.status, r.code()), (403, "not_available".into()));
}

#[test]
fn a_fixed_country_is_not_asked_and_not_contradicted() {
    let ts = start_with(|p| p.set_fixed_country(Some("JP")));
    let token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let v = get(&ts, "claim", Some(&token)).json();
    assert_eq!(v["country"]["fixed"], "JP");
    assert_eq!(v["country"]["choices"].as_array().unwrap().len(), 0);
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        &format!(
            r#"{{"country":"DE","guest":{{"name":"Cafe","mode":"open_to_all"}},"owner_password":"{PASSWORD}"}}"#
        ),
    );
    assert_eq!(r.json()["error"]["fields"][0]["code"], "not_allowed");
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        &format!(
            r#"{{"guest":{{"name":"Cafe","mode":"open_to_all"}},"owner_password":"{PASSWORD}"}}"#
        ),
    );
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["country"], "JP");
}

#[test]
fn check_reports_every_failing_field_and_persists_nothing() {
    let ts = start();
    let token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let long = "x".repeat(30);
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        &format!(
            r#"{{"country":"XX","guest":{{"name":"{long}","mode":"open_to_all"}},"owner_password":"password123"}}"#
        ),
    );
    assert_eq!((r.status, r.code()), (400, "invalid".into()));
    let fields: Vec<(String, String)> = r.json()["error"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["field"].as_str().unwrap().into(),
                f["code"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert!(
        fields.contains(&("country".into(), "not_in_choices".into())),
        "{fields:?}"
    );
    assert!(
        fields.contains(&("guest.name".into(), "too_long".into())),
        "{fields:?}"
    );
    assert!(
        fields.contains(&("owner_password".into(), "common_password".into())),
        "{fields:?}"
    );
    // A password equal to the guest name is refused (ENK-OWN-010).
    let r = post(
        &ts,
        "claim/check",
        Some(&token),
        r#"{"country":"JP","guest":{"name":"Cafe Example!","mode":"open_to_all"},"owner_password":"Cafe Example!"}"#,
    );
    assert_eq!(
        r.json()["error"]["fields"][0]["code"],
        "equals_network_name"
    );
    assert_eq!(get(&ts, "", None).json()["state"], "SETUP");
    assert!(
        !ts.platform
            .calls()
            .iter()
            .any(|c| c.starts_with("enter_pending")),
        "nothing changed on the device"
    );
    let r = post(&ts, "claim/confirm", Some(&token), "{}");
    assert_eq!((r.status, r.code()), (409, "no_proposal".into()));
}

#[test]
fn the_whole_claim() {
    let ts = start();
    let calls_before = ts.platform.calls().len();
    let (token, proposal) = claim(&ts);
    let key = proposal["management"]["key"].as_str().unwrap();
    assert!(is_management_key(key), "{key}");
    assert_eq!(proposal["management"]["name"], "Enkrnett-3F2A");
    assert_eq!(proposal["address"], "192.168.207.1");
    assert_eq!(proposal["state"], "SETUP");
    assert_eq!(proposal["final_in"], serde_json::Value::Null);
    let calls = ts.platform.calls();
    assert!(
        calls[calls_before..]
            .iter()
            .any(|c| c == "enter_pending(Enkrnett-3F2A)")
    );
    assert!(
        calls
            .last()
            .unwrap()
            .starts_with("apply_settings(v1, Cafe Example"),
        "{calls:?}"
    );
    // The owner sees the status, the key, the device.
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(status["state"], "CLAIMED");
    assert_eq!(status["settings"]["version"], 1);
    assert_eq!(status["settings"]["guest"]["name"], "Cafe Example");
    assert_eq!(
        status["settings"]["limits"]["management_name_max_bytes"],
        32
    );
    assert_eq!(status["guest"]["up"], true);
    assert_eq!(status["guest"]["clients_unencrypted"], 0);
    assert_eq!(status["management"]["name"], "Enkrnett-3F2A");
    assert_eq!(status["change"]["state"], "none");
    assert_eq!(status["change"]["id"], 0);
    assert_eq!(status["update"]["state"], "idle");
    assert_eq!(status["version"]["baseline"], "25.12");
    assert_eq!(status["hardening_met"], true);
    let mk = get(&ts, "management-key", Some(&token)).json();
    assert_eq!(mk["key"], key);
    let device = get(&ts, "device", Some(&token)).json();
    assert_eq!(device["device_id"].as_str().unwrap().len(), 16);
    assert_eq!(device["hardening"].as_array().unwrap().len(), 6);
    assert_eq!(device["version"]["api"], 1);
    // The greeting on the management network reports the session's protection.
    assert_eq!(get(&ts, "", None).json()["connection"]["security"], "wpa3");
}

#[test]
fn a_restart_in_pending_forgets_the_claim() {
    let ts = start();
    let claim_token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    post(
        &ts,
        "claim/check",
        Some(&claim_token),
        &format!(
            r#"{{"country":"JP","guest":{{"name":"Cafe","mode":"open_to_all"}},"owner_password":"{PASSWORD}"}}"#
        ),
    );
    let confirm = post(&ts, "claim/confirm", Some(&claim_token), "{}");
    assert_eq!(confirm.json()["state"], "PENDING");
    assert_eq!(confirm.json()["final_in"], 600);
    wait_for_state(&ts, "PENDING");
    assert_eq!(get(&ts, "claim", Some(&claim_token)).code(), "wrong_state");
    ts.app.do_restart();
    assert_eq!(get(&ts, "", None).json()["state"], "SETUP", "ENK-OWN-008");
    let r = post(
        &ts,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!(r.code(), "wrong_state");
}

#[test]
fn a_pending_claim_expires_after_ten_minutes() {
    let ts = start();
    let claim_token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    post(
        &ts,
        "claim/check",
        Some(&claim_token),
        &format!(
            r#"{{"country":"JP","guest":{{"name":"Cafe","mode":"open_to_all"}},"owner_password":"{PASSWORD}"}}"#
        ),
    );
    post(&ts, "claim/confirm", Some(&claim_token), "{}");
    wait_for_state(&ts, "PENDING");
    ts.clock.advance(Duration::from_secs(9 * 60));
    assert_eq!(get(&ts, "", None).json()["state"], "PENDING");
    ts.clock.advance(Duration::from_secs(61));
    assert_eq!(get(&ts, "", None).json()["state"], "SETUP");
    assert_eq!(ts.platform.calls().last().unwrap(), "enter_setup()");
}

#[test]
fn the_setup_window_closes_and_extends() {
    let ts = start();
    ts.clock.advance(Duration::from_secs(30 * 60 + 1));
    assert_eq!(get(&ts, "", None).json()["state"], "UNCLAIMED");
    assert_eq!(ts.platform.calls().last().unwrap(), "close_setup()");
    assert_eq!(post(&ts, "claim/start", None, "{}").code(), "wrong_state");

    // A claim session that is kept alive (a request at least every 5 minutes) earns the window
    // its one extension of 15 minutes when the 30 minutes end.
    let ts = start();
    let token = post(&ts, "claim/start", None, "{}").json()["token"]
        .as_str()
        .unwrap()
        .to_string();
    for _ in 0..8 {
        ts.clock.advance(Duration::from_secs(4 * 60));
        assert_eq!(get(&ts, "claim", Some(&token)).status, 200);
    }
    assert_eq!(
        get(&ts, "", None).json()["state"],
        "SETUP",
        "extended at the 30-minute mark"
    );
    ts.clock.advance(Duration::from_secs(12 * 60)); // 44 minutes
    assert_eq!(get(&ts, "", None).json()["state"], "SETUP");
    ts.clock.advance(Duration::from_secs(2 * 60)); // 46 minutes: past 30 + 15
    assert_eq!(get(&ts, "", None).json()["state"], "UNCLAIMED", "only once");
}

#[test]
fn login_delays_after_five_failures() {
    let ts = start();
    let _ = claim(&ts);
    for _ in 0..5 {
        let r = post(&ts, "login", None, r#"{"password":"wrong wrong wrong"}"#);
        assert_eq!((r.status, r.code()), (401, "wrong_password".into()));
    }
    let r = post(
        &ts,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!((r.status, r.code()), (429, "try_later".into()));
    assert_eq!(r.json()["error"]["retry_in"], 30);
    assert_eq!(
        r.header("Retry-After"),
        Some("30"),
        "ENK-OWN-012; chapter 9 §5"
    );
    ts.clock.advance(Duration::from_secs(31));
    let r = post(
        &ts,
        "login",
        None,
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!(r.status, 200);
}

#[test]
fn sessions_logout_and_at_most_four() {
    let ts = start();
    let (first, _) = claim(&ts);
    let mut tokens = vec![first];
    for _ in 0..4 {
        let r = post(
            &ts,
            "login",
            None,
            &format!(r#"{{"password":"{PASSWORD}"}}"#),
        );
        tokens.push(r.json()["token"].as_str().unwrap().to_string());
    }
    assert_eq!(
        get(&ts, "status", Some(&tokens[0])).status,
        401,
        "the fifth login ended the idle-longest session"
    );
    assert_eq!(get(&ts, "status", Some(&tokens[4])).status, 200);
    assert_eq!(post(&ts, "logout", Some(&tokens[4]), "{}").status, 200);
    assert_eq!(get(&ts, "status", Some(&tokens[4])).code(), "no_session");
    ts.clock.advance(Duration::from_secs(15 * 60 + 1));
    assert_eq!(
        get(&ts, "status", Some(&tokens[3])).code(),
        "no_session",
        "15 minutes idle"
    );
}

#[test]
fn settings_save_version_mismatch_and_undo() {
    let ts = start();
    let (token, _) = claim(&ts);
    let stale = post(
        &ts,
        "settings",
        Some(&token),
        r#"{"version":9,"guest":{"name":"New","mode":"open_to_all","enabled":true},"management":{"name":"Enkrnett-3F2A"}}"#,
    );
    assert_eq!(
        (stale.status, stale.code()),
        (409, "version_mismatch".into())
    );
    assert_eq!(stale.json()["error"]["settings"]["version"], 1);
    let bad = post(
        &ts,
        "settings",
        Some(&token),
        &format!(
            r#"{{"version":1,"guest":{{"name":"{}","mode":"open_to_all","enabled":true}},"management":{{"name":"Enkrnett-3F2A"}}}}"#,
            "x".repeat(30)
        ),
    );
    assert_eq!(bad.json()["error"]["fields"][0]["limit"], 29);
    let ok = post(
        &ts,
        "settings",
        Some(&token),
        r#"{"version":1,"guest":{"name":"Cafe Example 2","mode":"encrypted_only","enabled":true},"management":{"name":"Enkrnett-3F2A"}}"#,
    );
    assert_eq!(ok.status, 202, "{}", ok.body);
    let change = ok.json();
    assert_eq!(change["state"], "settled");
    assert_eq!(change["kind"], "settings");
    assert_eq!(change["needs_confirmation"], false);
    assert_eq!(
        (
            change["from_version"].as_u64(),
            change["to_version"].as_u64()
        ),
        (Some(1), Some(2))
    );
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(status["settings"]["version"], 2);
    assert_eq!(status["guest"]["mode"], "encrypted_only");
    assert_eq!(get(&ts, "change", Some(&token)).json()["id"], 1);
    // A failed health check undoes the change and reports why.
    ts.platform
        .fail_next_apply(enkrnett_model::api::HealthDetail::GuestDown);
    let undone = post(
        &ts,
        "settings",
        Some(&token),
        r#"{"version":2,"guest":{"name":"Third","mode":"open_to_all","enabled":true},"management":{"name":"Enkrnett-3F2A"}}"#,
    );
    assert_eq!(undone.status, 202);
    assert_eq!(undone.json()["state"], "undone");
    assert_eq!(undone.json()["reason"], "health_check_failed");
    assert_eq!(undone.json()["detail"], "guest_down");
    let status = get(&ts, "status", Some(&token)).json();
    assert_eq!(
        status["settings"]["version"], 2,
        "the previous settings are back"
    );
    assert_eq!(status["settings"]["guest"]["name"], "Cafe Example 2");
    // A management name change needs confirmation and is reported so.
    let mgmt = post(
        &ts,
        "settings",
        Some(&token),
        r#"{"version":2,"guest":{"name":"Cafe Example 2","mode":"encrypted_only","enabled":true},"management":{"name":"Office"}}"#,
    );
    assert_eq!(mgmt.json()["needs_confirmation"], true);
    // Restore is a change of its own kind.
    let restore = post(&ts, "settings/restore", Some(&token), "{}");
    assert_eq!(restore.json()["kind"], "restore");
}

#[test]
fn replacing_the_management_key_returns_the_compound_response() {
    let ts = start();
    let (token, proposal) = claim(&ts);
    let r = post(&ts, "management-key", Some(&token), "{}");
    assert_eq!(r.status, 202);
    let v = r.json();
    assert_eq!(v["kind"], "management_key");
    assert_eq!(v["needs_confirmation"], true);
    let new_key = v["management_key"]["key"].as_str().unwrap();
    assert!(is_management_key(new_key));
    assert_ne!(new_key, proposal["management"]["key"].as_str().unwrap());
    assert_eq!(
        get(&ts, "management-key", Some(&token)).json()["key"],
        new_key
    );
    assert_eq!(ts.platform.management().unwrap().1, new_key);
}

#[test]
fn changing_the_owner_password_checks_new_first_and_ends_sessions() {
    let ts = start();
    let (token, _) = claim(&ts);
    let r = post(
        &ts,
        "password",
        Some(&token),
        &format!(r#"{{"current":"wrong","new":"short"}}"#),
    );
    assert_eq!(
        (r.status, r.code()),
        (400, "invalid".into()),
        "the new value is checked before the current one"
    );
    assert_eq!(r.json()["error"]["fields"][0]["field"], "new");
    let r = post(
        &ts,
        "password",
        Some(&token),
        r#"{"current":"wrong wrong wrong","new":"another good password"}"#,
    );
    assert_eq!((r.status, r.code()), (401, "wrong_password".into()));
    let r = post(
        &ts,
        "password",
        Some(&token),
        &format!(r#"{{"current":"{PASSWORD}","new":"another good password"}}"#),
    );
    assert_eq!(r.status, 200);
    assert_eq!(
        get(&ts, "status", Some(&token)).code(),
        "no_session",
        "every session ended"
    );
    assert_eq!(
        post(
            &ts,
            "login",
            None,
            &format!(r#"{{"password":"{PASSWORD}"}}"#)
        )
        .status,
        401
    );
    assert_eq!(
        post(
            &ts,
            "login",
            None,
            r#"{"password":"another good password"}"#
        )
        .status,
        200
    );
}

#[test]
fn erase_needs_the_password_and_returns_the_device_to_setup() {
    let ts = start();
    let (token, _) = claim(&ts);
    let id_before = ts.app.device_id();
    let r = post(
        &ts,
        "erase",
        Some(&token),
        r#"{"password":"wrong wrong wrong"}"#,
    );
    assert_eq!((r.status, r.code()), (401, "wrong_password".into()));
    assert_eq!(get(&ts, "", None).json()["state"], "CLAIMED");
    let r = post(
        &ts,
        "erase",
        Some(&token),
        &format!(r#"{{"password":"{PASSWORD}"}}"#),
    );
    assert_eq!(r.status, 202);
    assert_eq!(r.body, "{}");
    wait_for_state(&ts, "SETUP");
    assert_ne!(
        ts.app.device_id(),
        id_before,
        "a new identifier (ENK-OWN-020)"
    );
    assert_eq!(
        get(&ts, "status", Some(&token)).code(),
        "wrong_state",
        "sessions and ownership are gone"
    );
    assert!(ts.platform.calls().contains(&"erase()".to_string()));
    assert_eq!(ts.platform.calls().last().unwrap(), "enter_setup()");
    // The device can be claimed again, with a fresh window.
    assert!(ts.app.window_ends_in() > 1700);
    assert_eq!(post(&ts, "claim/start", None, "{}").status, 200);
}

#[test]
fn restart_keeps_the_owner_and_ends_sessions() {
    let ts = start();
    let (token, _) = claim(&ts);
    let r = post(&ts, "restart", Some(&token), "{}");
    assert_eq!(r.status, 202);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(get(&ts, "", None).json()["state"], "CLAIMED");
    assert_eq!(get(&ts, "status", Some(&token)).code(), "no_session");
    assert_eq!(
        ts.platform
            .calls()
            .iter()
            .filter(|c| *c == "restart()")
            .count(),
        1
    );
}

#[test]
fn update_resources_in_this_stage() {
    let ts = start();
    let (token, _) = claim(&ts);
    let u = get(&ts, "update", Some(&token)).json();
    assert_eq!(u["state"], "idle");
    assert_eq!(u["checks_enabled"], true);
    assert_eq!(u["last_check"], serde_json::Value::Null);
    assert_eq!(u["can_undo"], false);
    let u = post(&ts, "update/check", Some(&token), "{}").json();
    assert_eq!(u["last_check"]["result"], "unreachable");
    assert_eq!(u["last_check"]["checked_ago"], 0);
    let r = post(&ts, "update/install", Some(&token), "{}");
    assert_eq!((r.status, r.code()), (409, "nothing_to_install".into()));
}

#[test]
fn the_connection_line_reports_wpa2() {
    let ts = start_with(|p| p.set_management_security(Some(ConnectionSecurity::Wpa2)));
    let _ = claim(&ts);
    assert_eq!(get(&ts, "", None).json()["connection"]["security"], "wpa2");
    let ts = start_with(|p| p.set_setup_security(None));
    assert_eq!(
        get(&ts, "", None).json()["connection"]["security"],
        serde_json::Value::Null
    );
}
