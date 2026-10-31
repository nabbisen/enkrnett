//! ENK-AUTH-002, 003, 005, 006; ENK-OWN-005.

use std::time::{Duration, Instant};

use enkrnett_model::api::SessionKind;

use super::*;

fn store_with(n: usize, now: Instant) -> (SessionStore, Vec<String>) {
    let mut s = SessionStore::new();
    let tokens: Vec<String> = (0..n)
        .map(|i| {
            s.create(SessionKind::Owner, format!("token-{i}"), now)
                .token
        })
        .collect();
    (s, tokens)
}

#[test]
fn authenticate_touches_and_unknown_is_none() {
    let now = Instant::now();
    let (mut s, t) = store_with(1, now);
    assert_eq!(s.authenticate(&t[0], now), Some(SessionKind::Owner));
    assert_eq!(s.authenticate("nope", now), None);
    assert_eq!(s.authenticate("token-", now), None);
}

#[test]
fn idle_and_life_limits() {
    let now = Instant::now();
    let (mut s, t) = store_with(1, now);
    let t14 = now + Duration::from_secs(14 * 60);
    assert!(s.authenticate(&t[0], t14).is_some());
    let t29 = t14 + Duration::from_secs(15 * 60);
    assert!(
        s.authenticate(&t[0], t29).is_none(),
        "15 minutes idle ends it"
    );

    let (mut s, t) = store_with(1, now);
    let mut tick = now;
    for _ in 0..(12 * 60 / 10) {
        tick += Duration::from_secs(10 * 60);
        s.authenticate(&t[0], tick - Duration::from_secs(1));
    }
    assert!(
        s.authenticate(&t[0], now + LIFE_LIMIT).is_none(),
        "12 hours ends it whatever the use"
    );
}

#[test]
fn at_most_four_the_idle_longest_goes_first() {
    let now = Instant::now();
    let (mut s, t) = store_with(4, now);
    s.authenticate(&t[0], now + Duration::from_secs(5));
    s.authenticate(&t[2], now + Duration::from_secs(6));
    s.authenticate(&t[3], now + Duration::from_secs(7));
    // t[1] is the one idle longest
    s.create(
        SessionKind::Owner,
        "token-4".into(),
        now + Duration::from_secs(8),
    );
    assert_eq!(s.len(), 4);
    assert!(
        s.authenticate(&t[1], now + Duration::from_secs(9))
            .is_none()
    );
    assert!(
        s.authenticate(&t[0], now + Duration::from_secs(9))
            .is_some()
    );
}

#[test]
fn claim_sessions_idle_out_after_five_minutes() {
    let now = Instant::now();
    let mut s = SessionStore::new();
    let c = s.create(SessionKind::Claim, "claim".into(), now);
    assert_eq!(c.idle_ends_in, 300);
    assert!(s.has_kind(SessionKind::Claim, now + Duration::from_secs(299)));
    assert!(!s.has_kind(SessionKind::Claim, now + Duration::from_secs(301)));
}

#[test]
fn ending_sessions() {
    let now = Instant::now();
    let (mut s, t) = store_with(3, now);
    s.end(&t[0]);
    assert_eq!(s.len(), 2);
    s.end_kind(SessionKind::Owner);
    assert!(s.is_empty());
    let (mut s, _) = store_with(2, now);
    s.end_all();
    assert!(s.is_empty());
}

#[test]
fn remaining_reports_both_limits() {
    let now = Instant::now();
    let (s, t) = store_with(1, now);
    assert_eq!(
        s.remaining(&t[0], now + Duration::from_secs(60)),
        Some((14 * 60, 12 * 3600 - 60))
    );
    assert_eq!(s.remaining("nope", now), None);
}
