//! The update (chapter 11 §2–3). The check: the device fetches the signed release index through
//! the platform's downloader, verifies the signature against the keys the image carries, refuses
//! an index older than one it has accepted, and offers only a release newer than the installed
//! one (ENK-UPD-001, ENK-UPD-008, ENK-UPD-009; ENK-RLS-005, ENK-RLS-006). By itself at most once
//! a day, and on request from the details screen. A development build, or a package an expert
//! installed over OpenWrt, does not look for updates unless the owner turns the checks on.
//!
//! The install (ENK-UPD-002 to 007), only when the owner presses *Update*: the image is
//! downloaded to RAM and checked one condition at a time (the device profile, the baseline, the
//! free memory before a byte arrives; the size, the SHA-512 and the signify signature over the
//! bytes, read once), the update marker goes to flash, and the image is handed to the
//! platform's root side, which verifies again and runs `sysupgrade` with the configuration
//! kept. The first boot of the new release finds the marker, judges itself by the start-up
//! health check, and records the outcome, which the details screen shows (ENK-UPD-005).

use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use enkrnett_model::api::{
    Available, CheckResult, ErrorCode, InstalledAs, UpdateFailure, UpdateState,
};
use enkrnett_model::release::{INDEX_MAX_BYTES, ReleaseEntry, ReleaseIndex, Version};
use enkrnett_model::signify::{self, MAX_TEXT, PublicKey, Sha512, Signature, Verifier};

use super::{ApiError, App, ChangeRecord, Inner, Reply};
use crate::platform::{FetchError, InstallVerdict};
use crate::store::UpdateMarker;
use crate::{log, random};

/// How long the root side has to answer an install request before the attempt counts as
/// failed; it copies and verifies the image meanwhile.
pub const VERDICT_WAIT: Duration = Duration::from_secs(120);
/// Once the root side says it is flashing, how long this process waits to be ended by the
/// restart before giving up on the attempt.
pub const FLASH_WAIT: Duration = Duration::from_secs(600);
const VERDICT_POLL: Duration = Duration::from_millis(500);
/// The staged files, by name, in `Config::staging_dir`; the root side knows the same names.
pub const IMAGE_FILE: &str = "image";
pub const SIGNATURE_FILE: &str = "image.sig";
/// The image is read once, in pieces of this size, for the hash and the signature.
const READ_CHUNK: usize = 64 * 1024;

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
    /// The index entry behind `available`.
    pub offer: Option<ReleaseEntry>,
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
                let offer = Version::parse(&self.0.config.version).and_then(|v| {
                    index
                        .offer(&self.0.device.profile, v, &self.0.device.baseline)
                        .cloned()
                });
                Checked {
                    result: CheckResult::Ok,
                    available: offer.as_ref().map(ReleaseEntry::available),
                    offer,
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
                    offer: None,
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
            inner.update.offer = checked.offer;
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

/// Why an attempt did not end in a flashed image: the reason for `last_result` and a word for
/// the log.
type Failed = (UpdateFailure, String);

impl App {
    fn image_path(&self) -> PathBuf {
        self.0.config.staging_dir.join(IMAGE_FILE)
    }

    fn signature_path(&self) -> PathBuf {
        self.0.config.staging_dir.join(SIGNATURE_FILE)
    }

    /// `update/install` (ENK-UPD-002): the owner pressed *Update*. The checks that need no bytes
    /// of the image answer at once (`cannot_update` with the reason, nothing written); the rest
    /// runs on its own thread while the shape says `downloading`, `verifying`, `installing`.
    pub fn update_install(&self, token: Option<&str>) -> super::Result<Reply> {
        let entry = {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
            if inner.update.state != UpdateState::Idle || inner.update.checking {
                return Err(ApiError::code(
                    ErrorCode::UpdateInProgress,
                    "an update or a check is in progress",
                ));
            }
            // ENK-UPD-003: no transaction may be pending (chapter 9 §5: with the change).
            if inner.change.as_ref().is_some_and(ChangeRecord::in_progress) {
                let mut d = enkrnett_model::api::ErrorDetail::new(ErrorCode::ChangePending)
                    .message("a change is in progress");
                d.change = Some(Self::change_shape(&inner, self.0.clock.now()));
                return Err(ApiError(Box::new(d)));
            }
            let entry = match (&inner.update.offer, self.checks_enabled(&inner)) {
                (Some(entry), true) => entry.clone(),
                _ => {
                    return Err(ApiError::code(
                        ErrorCode::NothingToInstall,
                        "no update is available",
                    ));
                }
            };
            let available_kib = self.0.platform.device_info().memory.available_kib;
            if let Err(reason) = entry.check_device(
                &self.0.device.profile,
                &self.0.device.baseline,
                available_kib,
            ) {
                log::event(
                    "update_failed",
                    &[
                        ("version", &entry.version),
                        ("reason", &reason.as_str()),
                        (
                            "why",
                            &format!("before download; available_kib={available_kib}"),
                        ),
                    ],
                );
                return Err(ApiError::cannot_update(
                    reason,
                    "refused before anything was written",
                ));
            }
            inner.update.state = UpdateState::Downloading;
            inner.update.last_result = None;
            entry
        };
        log::event("update_started", &[("version", &entry.version)]);
        let app = self.clone();
        std::thread::Builder::new()
            .name("enkrnettd-update-install".into())
            .spawn(move || app.run_install(entry))
            .expect("spawn update install");
        let inner = self.0.inner.lock().expect("app");
        let now = self.0.clock.now();
        Ok(Reply::accepted(&self.update_shape(&inner, now)))
    }

    /// The attempt, off the request thread: download, verify, hand over, wait for the verdict.
    fn run_install(&self, entry: ReleaseEntry) {
        let outcome = self
            .download_and_verify(&entry)
            .and_then(|()| self.hand_over(&entry));
        if let Err((reason, why)) = outcome {
            self.install_failed(&entry, reason, &why);
        }
    }

    fn set_state(&self, state: UpdateState) {
        self.0.inner.lock().expect("app").update.state = state;
    }

    /// Downloads the image and its signature to RAM and checks the bytes (ENK-UPD-003): the size
    /// and the SHA-512 the index states, and the signify signature against the keys the image
    /// carries, in one pass over the file.
    fn download_and_verify(&self, entry: &ReleaseEntry) -> Result<(), Failed> {
        let platform = &self.0.platform;
        let index_url = platform
            .update_index_url()
            .ok_or((UpdateFailure::DownloadFailed, "no index url".to_string()))?;
        let url = entry.image_url(&index_url);
        let staging = &self.0.config.staging_dir;
        let _ = fs::remove_dir_all(staging);
        fs::create_dir_all(staging)
            .map_err(|e| (UpdateFailure::DownloadFailed, format!("staging dir: {e}")))?;
        let image = self.image_path();
        let size = platform
            .download(&url, &image, entry.size)
            .map_err(|e| match e {
                FetchError::TooLarge => (
                    UpdateFailure::BadSignature,
                    "image larger than the index says".into(),
                ),
                e => (UpdateFailure::DownloadFailed, e.to_string()),
            })?;
        let signature_text = platform
            .fetch(&format!("{url}.sig"), MAX_TEXT)
            .map_err(|e| (UpdateFailure::DownloadFailed, format!("signature: {e}")))?;
        let signature_text = String::from_utf8(signature_text).map_err(|_| {
            (
                UpdateFailure::BadSignature,
                "signature: not text".to_string(),
            )
        })?;
        let signature = Signature::parse(&signature_text)
            .map_err(|e| (UpdateFailure::BadSignature, e.to_string()))?;
        fs::write(self.signature_path(), &signature_text).map_err(|e| {
            (
                UpdateFailure::DownloadFailed,
                format!("signature file: {e}"),
            )
        })?;
        self.set_state(UpdateState::Verifying);
        let keys: Vec<PublicKey> = platform
            .release_keys()
            .iter()
            .filter_map(|t| PublicKey::parse(t).ok())
            .collect();
        let mut verifier = Verifier::new(&keys, &signature).map_err(|e| {
            (
                UpdateFailure::BadSignature,
                format!("{e} ({} keys)", keys.len()),
            )
        })?;
        let mut hash = Sha512::new();
        let mut file = File::open(&image)
            .map_err(|e| (UpdateFailure::DownloadFailed, format!("image file: {e}")))?;
        let mut buf = vec![0u8; READ_CHUNK];
        let mut read_total: u64 = 0;
        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| (UpdateFailure::DownloadFailed, format!("image read: {e}")))?;
            if n == 0 {
                break;
            }
            read_total += n as u64;
            verifier.absorb(&buf[..n]);
            hash.absorb(&buf[..n]);
        }
        if read_total != size {
            return Err((
                UpdateFailure::BadSignature,
                "image changed while read".into(),
            ));
        }
        entry
            .check_bytes(size, &signify::hex(&hash.finish()))
            .map_err(|r| (r, format!("size {size} or sha512 differ from the index")))?;
        let key = verifier
            .finish()
            .map_err(|e| (UpdateFailure::BadSignature, e.to_string()))?;
        log::event(
            "image_verified",
            &[
                ("version", &entry.version),
                ("key", &signify::hex(&key.keynum)),
                ("size", &size),
            ],
        );
        Ok(())
    }

    /// The marker to flash, the state `installing`, the request to the root side, and its
    /// verdict. When the image is written this process ends with the restart and never returns
    /// from here; the first boot of the new release finishes the story.
    fn hand_over(&self, entry: &ReleaseEntry) -> Result<(), Failed> {
        let token = random::session_token();
        {
            let mut inner = self.0.inner.lock().expect("app");
            let marker = UpdateMarker::installing(
                &self.0.config.version,
                &entry.version,
                &entry.baseline,
                inner.store.data.owner.is_some(),
            );
            inner
                .store
                .write_update(&marker)
                .map_err(|e| (UpdateFailure::InstallFailed, format!("update marker: {e}")))?;
            inner.update.state = UpdateState::Installing;
        }
        log::event("update_installing", &[("version", &entry.version)]);
        self.0
            .platform
            .install(&self.image_path(), &self.signature_path(), &token)
            .map_err(|e| (UpdateFailure::InstallFailed, e.to_string()))?;
        let started = Instant::now();
        let mut flashing = false;
        loop {
            match self.0.platform.install_verdict(&token) {
                Some(InstallVerdict::Flashing) if !flashing => {
                    flashing = true;
                    log::event("update_flashing", &[("version", &entry.version)]);
                }
                Some(InstallVerdict::Flashing) => {}
                Some(InstallVerdict::Refused(reason)) => {
                    return Err((reason, "refused by the root side".into()));
                }
                Some(InstallVerdict::Failed) => {
                    return Err((
                        UpdateFailure::InstallFailed,
                        "sysupgrade did not succeed".into(),
                    ));
                }
                None => {}
            }
            let limit = if flashing { FLASH_WAIT } else { VERDICT_WAIT };
            if started.elapsed() >= limit {
                return Err((
                    UpdateFailure::InstallFailed,
                    if flashing {
                        "still running after the flash".into()
                    } else {
                        "no verdict from the root side".into()
                    },
                ));
            }
            std::thread::sleep(VERDICT_POLL);
        }
    }

    /// Nothing was written (or the written image did not take): the outcome for the shape and
    /// the marker, the state idle, the staged files gone (ENK-API-029: one line).
    fn install_failed(&self, entry: &ReleaseEntry, reason: UpdateFailure, why: &str) {
        let _ = fs::remove_dir_all(&self.0.config.staging_dir);
        let mut inner = self.0.inner.lock().expect("app");
        let marker = UpdateMarker::installing(
            &self.0.config.version,
            &entry.version,
            &entry.baseline,
            inner.store.data.owner.is_some(),
        )
        .failed(reason);
        if let Err(e) = inner.store.write_update(&marker) {
            log::event("store_failed", &[("at", &"update_marker"), ("error", &e)]);
        }
        inner.update.last_result = marker.last_result();
        inner.update.state = UpdateState::Idle;
        log::event(
            "update_failed",
            &[
                ("version", &entry.version),
                ("reason", &reason.as_str()),
                ("why", &why),
            ],
        );
    }

    /// The first boot after an update (ENK-UPD-005): the marker says `installing`. The carried
    /// state has been read (a format this release cannot read would have stopped the daemon
    /// before here, ENK-BASE-005); the running version must be the one handed over, the owner
    /// must still be there, and the start-up health check must have passed. Then the update is
    /// done; otherwise it is failed, and the device runs on with what it has.
    pub(super) fn finish_update_after_boot(&self, marker: UpdateMarker, healthy: bool) {
        let running = self.0.config.version.as_str();
        let mut inner = self.0.inner.lock().expect("app");
        log::event(
            "update_first_boot",
            &[
                ("from", &marker.from_version),
                ("to", &marker.to_version),
                ("running", &running),
                ("store_format", &inner.store.data.format),
            ],
        );
        let verdict: Result<(), Failed> = if running != marker.to_version {
            Err((
                UpdateFailure::InstallFailed,
                format!("running {running}, not {}", marker.to_version),
            ))
        } else if marker.had_owner && inner.store.data.owner.is_none() {
            Err((
                UpdateFailure::HealthCheckFailed,
                "the owner record was not carried".into(),
            ))
        } else if !healthy {
            Err((
                UpdateFailure::HealthCheckFailed,
                "the start-up health check failed".into(),
            ))
        } else {
            Ok(())
        };
        let marker = match &verdict {
            Ok(()) => {
                log::event(
                    "update_done",
                    &[
                        ("version", &marker.to_version),
                        ("from", &marker.from_version),
                    ],
                );
                marker.done()
            }
            Err((reason, why)) => {
                log::event(
                    "update_failed",
                    &[
                        ("version", &marker.to_version),
                        ("reason", &reason.as_str()),
                        ("why", &why),
                    ],
                );
                marker.failed(*reason)
            }
        };
        if let Err(e) = inner.store.write_update(&marker) {
            log::event("store_failed", &[("at", &"update_marker"), ("error", &e)]);
        }
        inner.update.last_result = marker.last_result();
    }
}

#[cfg(test)]
mod tests;
