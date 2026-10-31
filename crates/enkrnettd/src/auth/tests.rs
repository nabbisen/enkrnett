//! ENK-OWN-011, ENK-OWN-012.

use std::time::{Duration, Instant};

use super::*;

fn small_hasher() -> Hasher {
    // Small parameters keep the test fast; the device uses the profile's (ENK-FOOT-017).
    Hasher::new(64, 1, 1)
}

#[test]
fn hash_verifies_and_differs_per_salt() {
    let h = small_hasher();
    let a = h.hash("correct horse battery");
    let b = h.hash("correct horse battery");
    assert_ne!(a, b);
    assert!(a.starts_with("$argon2id$"));
    assert!(h.verify("correct horse battery", &a));
    assert!(!h.verify("correct horse batter", &a));
    assert!(!h.verify("anything", "not a hash"));
    assert!(!a.contains("correct horse"));
}

#[test]
fn delay_starts_at_the_fifth_failure_and_doubles() {
    let t0 = Instant::now();
    let mut g = LoginGuard::default();
    for _ in 0..4 {
        g.failure(t0);
        assert!(g.check(t0).is_ok());
    }
    g.failure(t0); // fifth
    assert_eq!(g.check(t0), Err(30));
    assert!(g.check(t0 + Duration::from_secs(30)).is_ok());
    g.failure(t0 + Duration::from_secs(30)); // sixth: 60 s
    assert_eq!(g.check(t0 + Duration::from_secs(30)), Err(60));
    g.failure(t0 + Duration::from_secs(90)); // seventh: 120 s
    assert_eq!(g.check(t0 + Duration::from_secs(90)), Err(120));
}

#[test]
fn delay_is_capped_at_fifteen_minutes() {
    let t0 = Instant::now();
    let mut g = LoginGuard::default();
    for _ in 0..40 {
        g.failure(t0);
    }
    assert_eq!(g.check(t0), Err(900));
}

#[test]
fn success_ends_the_delay() {
    let t0 = Instant::now();
    let mut g = LoginGuard::default();
    for _ in 0..6 {
        g.failure(t0);
    }
    assert!(g.check(t0).is_err());
    g.success();
    assert!(g.check(t0).is_ok());
    assert_eq!(g.consecutive_failures(), 0);
}

#[test]
fn retry_in_counts_down_and_never_says_zero_while_refusing() {
    let t0 = Instant::now();
    let mut g = LoginGuard::default();
    for _ in 0..5 {
        g.failure(t0);
    }
    assert_eq!(g.check(t0 + Duration::from_millis(29_500)), Err(1));
}
