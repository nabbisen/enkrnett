//! ENK-UPD-001: the daily spacing of the automatic check.

use super::*;

#[test]
fn the_daily_check_is_due_once_a_day_by_a_plausible_clock() {
    let t = PLAUSIBLE_CLOCK + 1_000_000;
    assert!(auto_check_due(None, t));
    assert!(!auto_check_due(Some(t), t + 1));
    assert!(!auto_check_due(Some(t), t + AUTO_CHECK_INTERVAL_SECS - 1));
    assert!(auto_check_due(Some(t), t + AUTO_CHECK_INTERVAL_SECS));
    // The clock went backwards (set by NTP after a check): due again rather than stuck.
    assert!(auto_check_due(Some(t), t - 1));
    // 1970: the clock is not set; the automatic check waits, whatever was recorded.
    assert!(!auto_check_due(None, 120));
    assert!(!auto_check_due(Some(0), 120));
}
