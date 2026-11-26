//! The update check (chapter 11 §2–3: ENK-UPD-001, ENK-UPD-008, ENK-UPD-009; ENK-RLS-005,
//! ENK-RLS-006). The device fetches the signed release index through the platform's downloader,
//! verifies the signature against the keys the image carries, refuses an index older than one
//! it has accepted, and offers only a release newer than the installed one. By itself at most
//! once a day, and on request from the details screen. A development build, or a package an
//! expert installed over OpenWrt, does not look for updates unless the owner turns the checks
//! on. Installing (chapter 11 §3, the rest) is the next stage.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use enkrnett_model::api::{Available, CheckResult, InstalledAs, UpdateState};
use enkrnett_model::release::{INDEX_MAX_BYTES, ReleaseIndex, Version};
use enkrnett_model::signify::{self, MAX_TEXT, PublicKey, Signature};

use super::{App, Inner};
use crate::log;
use crate::platform::FetchError;

/// ENK-UPD-001: by itself at most once a day.
pub const AUTO_CHECK_INTERVAL_SECS: u64 = 86_400;
/// The first check of a boot waits for the uplink and the clock to settle.
pub const AUTO_CHECK_FIRST_DELAY: Duration = Duration::from_secs(120);
/// A clock earlier than this has not been set (a router without a battery boots in 1970); the
/// daily spacing cannot be judged by it, so the automatic check waits for a plausible time.
pub const PLAUSIBLE_CLOCK: u64 = 1_735_689_600; // 2025-01-01T00:00:00Z

/// Whether the daily check is due: never done, a day has passed, or the clock went backwards.
pub fn auto_check_due(last: Option<u64>, now: u64) -> bool {
    if now < PLAUSIBLE_CLOCK {
        return false;
    }
    match last {
        None => true,
        Some(last) => now < last || now - last >= AUTO_CHECK_INTERVAL_SECS,
    }
}

/// The outcome of one check: the result for the shape and, when the index was accepted, its
/// sequence and the offer.
pub(super) struct Checked {
    pub result: CheckResult,
    pub available: Option<Available>,
    pub sequence: Option<u64>,
}

/// Why a check did not end in an accepted index: the result for the shape and a word for the log.
type Refused = (CheckResult, String);

fn refused<E: std::fmt::Display>(result: CheckResult) -> impl Fn(E) -> Refused {
    move |e| (result, e.to_string())
}

impl App {
    /// ENK-UPD-008: the owner's switch, else "installed from a published image and not a
    /// development build".
    pub(super) fn checks_enabled(&self, inner: &Inner) -> bool {
        match inner.store.data.update_checks {
            Some(on) => on,
            None => !self.0.config.development && self.0.device.installed_as == InstalledAs::Image,
        }
    }

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Fetches the index and its signature, verifies, parses, and compares the sequence.
    fn fetch_index(&self, known_sequence: u64) -> Result<ReleaseIndex, Refused> {
        let platform = &self.0.platform;
        let url = platform
            .update_index_url()
            .ok_or_else(|| (CheckResult::Unreachable, "no index url".to_string()))?;
        let keys: Vec<PublicKey> = platform
            .release_keys()
            .iter()
            .filter_map(|t| PublicKey::parse(t).ok())
            .collect();
        let index_bytes = platform.fetch(&url, INDEX_MAX_BYTES).map_err(|e| match e {
            FetchError::TooLarge => (CheckResult::BadIndex, "index too large".to_string()),
            e => (CheckResult::Unreachable, e.to_string()),
        })?;
        let signature = platform
            .fetch(&format!("{url}.sig"), MAX_TEXT)
            .map_err(|e| match e {
                FetchError::TooLarge => (CheckResult::BadSignature, "signature too large".into()),
                e => (CheckResult::Unreachable, format!("signature: {e}")),
            })?;
        let signature = std::str::from_utf8(&signature)
            .map_err(refused(CheckResult::BadSignature))
            .and_then(|t| Signature::parse(t).map_err(refused(CheckResult::BadSignature)))?;
        let key = signify::verify(&keys, &index_bytes, &signature).map_err(|e| {
            (
                CheckResult::BadSignature,
                format!("{e} ({} keys)", keys.len()),
            )
        })?;
        let index = ReleaseIndex::parse(&index_bytes).map_err(refused(CheckResult::BadIndex))?;
        if index.sequence < known_sequence {
            return Err((
                CheckResult::StaleIndex,
                format!("sequence {} below {known_sequence}", index.sequence),
            ));
        }
        log::event(
            "index_verified",
            &[
                ("key", &signify::hex(&key.keynum)),
                ("sequence", &index.sequence),
            ],
        );
        Ok(index)
    }

    /// One check, outside the lock (the network takes its time).
    pub(super) fn run_update_check(&self) -> Checked {
        let known_sequence = self.0.inner.lock().expect("app").store.data.index_sequence;
        match self.fetch_index(known_sequence) {
            Ok(index) => {
                let available = Version::parse(&self.0.config.version).and_then(|v| {
                    index
                        .offer(&self.0.device.profile, v, &self.0.device.baseline)
                        .map(|r| r.available())
                });
                Checked {
                    result: CheckResult::Ok,
                    available,
                    sequence: Some(index.sequence),
                }
            }
            Err((result, why)) => {
                log::event(
                    "update_check_failed",
                    &[("result", &result.as_str()), ("why", &why)],
                );
                Checked {
                    result,
                    available: None,
                    sequence: None,
                }
            }
        }
    }

    /// Records a check's outcome (ENK-API-029: one log line per check). A refused index changes
    /// nothing but the last check.
    pub(super) fn record_check(&self, inner: &mut Inner, checked: Checked, by: &str) {
        inner.update.checking = false;
        inner.update.last_check = Some((self.0.clock.now(), checked.result));
        if checked.result == CheckResult::Ok {
            inner.update.available = checked.available;
            if let Some(seq) = checked.sequence
                && seq > inner.store.data.index_sequence
            {
                inner.store.data.index_sequence = seq;
                self.save_store(inner, "index_sequence");
            }
        }
        let available = inner
            .update
            .available
            .as_ref()
            .map_or("none", |a| a.version.as_str());
        log::event(
            "update_checked",
            &[
                ("by", &by),
                ("result", &checked.result.as_str()),
                ("available", &available),
            ],
        );
    }

    pub(super) fn save_store(&self, inner: &mut Inner, at: &str) {
        if let Err(e) = inner.store.save() {
            log::event("store_failed", &[("at", &at), ("error", &e)]);
        }
    }

    /// The daily check (ENK-UPD-001), from the tick thread: when the device is claimed, checks
    /// are on, nothing else is running, and a day has passed by a plausible clock.
    pub fn check_update_if_due(&self) {
        {
            let mut inner = self.0.inner.lock().expect("app");
            let now = self.0.clock.now();
            if now.saturating_duration_since(self.0.started) < AUTO_CHECK_FIRST_DELAY
                || inner.store.data.owner.is_none()
                || !self.checks_enabled(&inner)
                || inner.update.checking
                || inner.update.state != UpdateState::Idle
            {
                return;
            }
            let unix = Self::now_unix();
            if !auto_check_due(inner.store.data.last_auto_check, unix) {
                return;
            }
            inner.update.checking = true;
            inner.store.data.last_auto_check = Some(unix);
            self.save_store(&mut inner, "last_auto_check");
        }
        let app = self.clone();
        std::thread::Builder::new()
            .name("enkrnettd-update-check".into())
            .spawn(move || {
                let checked = app.run_update_check();
                let mut inner = app.0.inner.lock().expect("app");
                app.record_check(&mut inner, checked, "daily");
            })
            .expect("spawn update check");
    }
}

#[cfg(test)]
mod tests;
