//! The effective state and drift (ENK-CFG-013, ENK-CFG-014, ENK-API-027, ENK-TOPO-003).
//!
//! `status` answers from readings at most 2 s old, whatever the number of clients polling. The
//! owned sections are compared with what the daemon last wrote at start-up, every 10 minutes,
//! and, while the status is being watched, at most every 30 s; a difference is drift, reported
//! and never overwritten by the daemon itself. The same check notices an uplink that has come
//! to collide with a Wi-Fi network and renumbers that network.

use std::time::{Duration, Instant};

use crate::app::change::ChangeRecord;
use crate::clock::seconds_since;
use crate::log;
use crate::platform::{Effective, PlatformError};

use super::{App, Inner};

/// How old a reading may be when `status` answers (ENK-API-027).
pub const READINGS_MAX_AGE: Duration = Duration::from_secs(2);
/// The drift check of the tick thread (ENK-CFG-014: at least every 10 minutes).
pub const DRIFT_INTERVAL: Duration = Duration::from_secs(600);
/// The drift check `status` may trigger while the UI is open.
pub const STATUS_DRIFT_INTERVAL: Duration = Duration::from_secs(30);
/// How often the uplink's subnet is compared with the networks (ENK-TOPO-003: one cheap ubus
/// call; the uplink's lease may arrive after the daemon started).
pub const UPLINK_INTERVAL: Duration = Duration::from_secs(10);

pub(super) struct Readings {
    pub at: Instant,
    pub effective: Effective,
    pub hardening_met: bool,
}

impl App {
    /// The effective state, read from the platform when the cached one is older than 2 s.
    pub(super) fn readings<'a>(&self, inner: &'a mut Inner, now: Instant) -> &'a Readings {
        let stale = inner
            .readings
            .as_ref()
            .is_none_or(|r| now.saturating_duration_since(r.at) >= READINGS_MAX_AGE);
        if stale {
            let effective = self.0.platform.effective();
            let hardening_met = self
                .0
                .platform
                .device_info()
                .hardening
                .iter()
                .all(|h| h.met);
            inner.readings = Some(Readings {
                at: now,
                effective,
                hardening_met,
            });
        }
        inner.readings.as_ref().expect("just set")
    }

    /// Runs the drift and collision check when the last one is older than `interval` and no
    /// change is running. The platform is read outside the lock.
    pub fn check_drift_if_due(&self, interval: Duration) {
        {
            let mut inner = self.0.inner.lock().expect("app");
            let now = self.0.clock.now();
            let claimed = inner.store.data.owner.is_some();
            let busy = inner.change.as_ref().is_some_and(ChangeRecord::in_progress);
            let recent = inner
                .drift_checked
                .is_some_and(|t| now.saturating_duration_since(t) < interval);
            if !claimed || busy || recent {
                return;
            }
            inner.drift_checked = Some(now);
        }
        let drift = match self.0.platform.owned_drift() {
            Ok(d) => d,
            Err(e) => {
                self.platform_failed("owned_drift", &e);
                return;
            }
        };
        let mut inner = self.0.inner.lock().expect("app");
        if drift && !inner.drift {
            log::event("drift_detected", &[]);
        }
        inner.drift = drift;
    }

    /// Runs the uplink collision check when the last one is older than `interval` and no
    /// change is running (ENK-TOPO-003).
    pub fn check_uplink_if_due(&self, interval: Duration) {
        {
            let mut inner = self.0.inner.lock().expect("app");
            let now = self.0.clock.now();
            let claimed = inner.store.data.owner.is_some();
            let busy = inner.change.as_ref().is_some_and(ChangeRecord::in_progress);
            let recent = inner
                .uplink_checked
                .is_some_and(|t| now.saturating_duration_since(t) < interval);
            if !claimed || busy || recent {
                return;
            }
            inner.uplink_checked = Some(now);
        }
        if self.0.platform.uplink_collides() {
            self.renumber();
        }
    }

    /// ENK-TOPO-003: the uplink's subnet collides with a Wi-Fi network; the platform renumbers
    /// the colliding one while re-applying the stored document. Not an owner-visible change.
    fn renumber(&self) {
        let record = {
            let inner = self.0.inner.lock().expect("app");
            inner.store.data.owner.clone()
        };
        let Some(record) = record else {
            return;
        };
        log::event("renumbering", &[]);
        match self.0.platform.apply_settings(
            &record.management_name,
            &record.management_key,
            &record.settings,
        ) {
            Ok(details) if details.is_empty() => {}
            Ok(details) => log::event(
                "renumber_health_check_failed",
                &[("detail", &format!("{details:?}"))],
            ),
            Err(e) => self.platform_failed("renumber", &e),
        }
        self.follow_management_address();
    }

    /// When the management address moved away from the one the listener is bound to, the
    /// daemon asks for its service to be restarted: the new process binds the new address
    /// (ENK-TOPO-006). Before the listener exists there is nothing to follow.
    pub(super) fn follow_management_address(&self) {
        let Some(bound) = self.bound() else {
            return;
        };
        let wanted = self.listen_address();
        if wanted.ip() != bound.ip() {
            log::event(
                "listen_address_changed",
                &[("from", &bound.ip()), ("to", &wanted.ip())],
            );
            self.leave_for_restart();
        }
    }

    /// Asks the service manager for a restart, then ends this process. If the request fails,
    /// the daemon stays up and says so: a plain exit would only be respawned unchanged.
    fn leave_for_restart(&self) {
        match self.0.platform.restart_service() {
            Ok(()) => {
                log::event("service_restart", &[("action", &"requested; exiting")]);
                std::process::exit(0);
            }
            Err(e) => log::event(
                "service_restart_failed",
                &[("error", &e), ("action", &"staying up")],
            ),
        }
    }

    /// Logs a platform failure; a file that is not ours any more ends the process, which the
    /// service manager restarts with ownership restored by the init script.
    pub(super) fn platform_failed(&self, at: &str, e: &PlatformError) {
        match e {
            PlatformError::NotOurs(m) => {
                log::event("files_not_ours", &[("at", &at), ("error", &m)]);
                self.leave_for_restart();
            }
            PlatformError::Failed(_) => {
                log::event("platform_failed", &[("at", &at), ("error", &e)]);
            }
        }
    }

    /// Seconds since the readings were taken, for `checked_ago`.
    pub(super) fn readings_age(inner: &Inner, now: Instant) -> u64 {
        inner
            .readings
            .as_ref()
            .map(|r| seconds_since(r.at, now))
            .unwrap_or(0)
    }
}
