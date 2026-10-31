//! ENK-OWN-001, 002, 003, 008 and the transition table of chapter 4 §1.

use std::time::{Duration, Instant};

use enkrnett_model::settings::{GuestMode, GuestSettings, ManagementSettings, Settings};

use super::*;

fn record() -> OwnerRecord {
    OwnerRecord {
        password_hash: "h".into(),
        management_name: "Enkrnett-3F2A".into(),
        management_key: "k".into(),
        settings: Settings {
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
        },
    }
}

const MIN: Duration = Duration::from_secs(60);

#[test]
fn starts_in_setup_without_an_owner_and_claimed_with_one() {
    let t0 = Instant::now();
    let (o, e) = Ownership::start(false, t0);
    assert_eq!(o.state(), OwnershipState::Setup);
    assert_eq!(e, Some(Event::SetupOpened));
    assert_eq!(o.window_ends_in(t0), 30 * 60);
    let (o, e) = Ownership::start(true, t0);
    assert_eq!(o.state(), OwnershipState::Claimed);
    assert_eq!(e, None);
    assert_eq!(o.window_ends_in(t0), 0);
}

#[test]
fn the_window_closes_after_thirty_minutes_without_a_claim() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(false, t0);
    assert!(o.tick(t0 + 29 * MIN, false).is_empty());
    assert_eq!(o.tick(t0 + 30 * MIN, false), vec![Event::SetupClosed]);
    assert_eq!(o.state(), OwnershipState::Unclaimed);
    assert!(
        o.tick(t0 + 60 * MIN, true).is_empty(),
        "nothing reopens the window but power"
    );
}

#[test]
fn a_claim_under_way_extends_the_window_once() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(false, t0);
    assert!(o.tick(t0 + 30 * MIN, true).is_empty());
    assert_eq!(o.state(), OwnershipState::Setup);
    assert_eq!(o.window_ends_in(t0 + 30 * MIN), 15 * 60);
    assert_eq!(
        o.tick(t0 + 45 * MIN, true),
        vec![Event::SetupClosed],
        "only once"
    );
}

#[test]
fn confirm_then_final() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(false, t0);
    assert!(o.confirm(record(), t0 + 5 * MIN).is_ok());
    assert_eq!(o.state(), OwnershipState::Pending);
    assert_eq!(o.final_in(t0 + 5 * MIN), Some(600));
    assert!(o.tick(t0 + 14 * MIN, false).is_empty());
    let r = o.make_final().expect("record");
    assert_eq!(r.management_name, "Enkrnett-3F2A");
    assert_eq!(o.state(), OwnershipState::Claimed);
    assert!(o.make_final().is_none());
}

#[test]
fn a_pending_claim_expires_and_the_window_continues() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(false, t0);
    o.confirm(record(), t0 + 20 * MIN).unwrap();
    assert!(o.tick(t0 + 29 * MIN, false).is_empty());
    assert_eq!(o.tick(t0 + 30 * MIN, false), vec![Event::PendingExpired]);
    assert_eq!(o.state(), OwnershipState::Setup);
    assert!(o.pending().is_none());
    // 10 minutes in PENDING did not count: 10 minutes of the window remain.
    assert_eq!(o.window_ends_in(t0 + 30 * MIN), 10 * 60);
    // Having returned from PENDING earns the one extension.
    assert!(o.tick(t0 + 40 * MIN, false).is_empty());
    assert_eq!(o.state(), OwnershipState::Setup);
    assert_eq!(o.tick(t0 + 55 * MIN, false), vec![Event::SetupClosed]);
}

#[test]
fn confirm_is_refused_outside_setup() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(true, t0);
    assert_eq!(o.confirm(record(), t0), Err(OwnershipState::Claimed));
}

#[test]
fn erase_leads_to_unclaimed() {
    let t0 = Instant::now();
    let (mut o, _) = Ownership::start(true, t0);
    o.erase();
    assert_eq!(o.state(), OwnershipState::Unclaimed);
    assert!(o.tick(t0 + MIN, false).is_empty());
}
