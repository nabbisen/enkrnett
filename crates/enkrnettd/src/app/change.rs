//! The transaction of chapter 6 §4 (ENK-CFG-005 to 012, ENK-API-021 to 023).
//!
//! ```text
//! start_change: validate ─► marker on flash ─► state applying ─► 202
//!                                                 │ (own thread)
//! perform:      apply + health check ─► ok, no confirmation ──► settle
//!                                     ─► ok, confirmation ─────► disconnect stations ─► waiting
//!                                     ─► failed ───────────────► undo (health_check_failed)
//! note_owner_request: first owner request on a connection accepted after the disconnection,
//!               while waiting ─────────────────────────────────► settle
//! tick:         waiting for 600 s ──────────────────────────────► undo (not_confirmed)
//! App::new:     marker found ───────────────────────────────────► undone (restart); the stored
//!               document, which the marker's snapshot equals, is applied as at every start
//! ```
//!
//! One transaction at a time (ENK-CFG-011). Every duration is on the monotonic clock
//! (ENK-CFG-012). The store is saved only when a change settles, so what is on flash is always
//! the last settled document, which is the snapshot the undo restores.

use std::time::{Duration, Instant};

use enkrnett_model::STORAGE_FORMAT;
use enkrnett_model::api::{
    Change, ChangeKind, ChangeState, ErrorCode, ErrorDetail, HealthDetail, SessionKind, UndoReason,
};
use enkrnett_model::settings::Settings;

use super::{ApiError, App, Inner, Result};
use crate::clock::{seconds_since, seconds_until};
use crate::log;
use crate::store::{OwnerRecord, PendingMarker};

/// How long the owner has to join the new management network (ENK-CFG-008; RFC 016).
pub const CONFIRMATION_LIMIT: Duration = Duration::from_secs(600);

/// The transaction in progress or the last one, as the daemon sees it.
pub(super) struct ChangeRecord {
    pub shape: Change,
    pub started: Instant,
    /// The document the change installs when it settles; `None` once decided.
    candidate: Option<OwnerRecord>,
    /// The document to go back to on undo.
    previous: OwnerRecord,
    /// When the management stations were disconnected (ENK-API-021).
    disconnected_at: Option<Instant>,
    /// When a waiting change is undone unless confirmed.
    confirm_deadline: Option<Instant>,
    /// The undo is running on its own thread; the outcome is fixed.
    undoing: bool,
}

impl ChangeRecord {
    /// The record of a change found undone at start-up (ENK-CFG-009).
    pub(super) fn undone_by_restart(marker: PendingMarker, now: Instant) -> ChangeRecord {
        ChangeRecord {
            shape: Change {
                id: marker.id,
                state: ChangeState::Undone,
                kind: Some(marker.kind),
                from_version: Some(marker.from_version),
                to_version: Some(marker.to_version),
                needs_confirmation: Some(marker.needs_confirmation),
                confirm_in: None,
                reason: Some(UndoReason::Restart),
                detail: None,
                started_ago: Some(0),
            },
            started: now,
            candidate: None,
            previous: marker.previous,
            disconnected_at: None,
            confirm_deadline: None,
            undoing: false,
        }
    }

    /// A restore cut short by a restart: the start-up apply of the stored document is the
    /// restore, so the change is reported settled.
    pub(super) fn settled_by_restart(marker: PendingMarker, now: Instant) -> ChangeRecord {
        let mut c = ChangeRecord::undone_by_restart(marker, now);
        c.shape.state = ChangeState::Settled;
        c.shape.reason = None;
        c
    }

    pub(super) fn in_progress(&self) -> bool {
        matches!(
            self.shape.state,
            ChangeState::Validating | ChangeState::Applying | ChangeState::WaitingConfirmation
        )
    }
}

impl App {
    /// The `change` shape with its live counters.
    pub(super) fn change_shape(inner: &Inner, now: Instant) -> Change {
        match &inner.change {
            None => Change::none(),
            Some(c) => {
                let mut shape = c.shape.clone();
                shape.started_ago = Some(seconds_since(c.started, now));
                shape.confirm_in = match (c.shape.state, c.confirm_deadline) {
                    (ChangeState::WaitingConfirmation, Some(deadline)) => {
                        Some(seconds_until(now, deadline))
                    }
                    _ => None,
                };
                shape
            }
        }
    }

    /// Starts a change: validated by the caller, refused while another runs, marker on flash
    /// first, then applied on its own thread. Returns the `applying` shape for the 202.
    pub(super) fn start_change(
        &self,
        inner: &mut Inner,
        kind: ChangeKind,
        new_settings: Settings,
        new_key: Option<String>,
    ) -> Result<Change> {
        if inner.change.as_ref().is_some_and(ChangeRecord::in_progress) {
            let mut d =
                ErrorDetail::new(ErrorCode::ChangePending).message("a change is in progress");
            d.change = Some(Self::change_shape(inner, self.0.clock.now()));
            return Err(ApiError(Box::new(d)));
        }
        let previous = inner
            .store
            .data
            .owner
            .clone()
            .expect("claimed has an owner");
        let now = self.0.clock.now();
        let needs_confirmation =
            new_settings.management.name != previous.management_name || new_key.is_some();
        let from_version = previous.settings.version;
        // A restore rewrites the device from the document it already has: the version stays.
        let to_version = match kind {
            ChangeKind::Restore => from_version,
            _ => from_version + 1,
        };
        let mut settings = new_settings;
        settings.version = to_version;
        let candidate = OwnerRecord {
            password_hash: previous.password_hash.clone(),
            management_name: settings.management.name.clone(),
            management_key: new_key.unwrap_or_else(|| previous.management_key.clone()),
            settings,
        };
        let id = inner.change_counter + 1;
        // ENK-CFG-006: the marker is on flash before the device is touched.
        inner
            .store
            .write_pending(&PendingMarker {
                format: STORAGE_FORMAT,
                id,
                kind,
                from_version,
                to_version,
                needs_confirmation,
                previous: previous.clone(),
            })
            .map_err(|e| ApiError::code(ErrorCode::Internal, &e.to_string()))?;
        inner.change_counter = id;
        let shape = Change {
            id,
            state: ChangeState::Applying,
            kind: Some(kind),
            from_version: Some(from_version),
            to_version: Some(to_version),
            needs_confirmation: Some(needs_confirmation),
            confirm_in: None,
            reason: None,
            detail: None,
            started_ago: Some(0),
        };
        inner.change = Some(ChangeRecord {
            shape: shape.clone(),
            started: now,
            candidate: Some(candidate.clone()),
            previous,
            disconnected_at: None,
            confirm_deadline: None,
            undoing: false,
        });
        log::event(
            "change_started",
            &[
                ("id", &id),
                ("kind", &format!("{kind:?}")),
                ("needs_confirmation", &needs_confirmation),
            ],
        );
        let app = self.clone();
        std::thread::Builder::new()
            .name(format!("enkrnettd-change-{id}"))
            .spawn(move || app.perform_change(id, candidate, needs_confirmation))
            .expect("spawn change");
        Ok(shape)
    }

    /// The part of the change that touches the device: apply, health check, then the decision.
    fn perform_change(&self, id: u64, candidate: OwnerRecord, needs_confirmation: bool) {
        let outcome = self.0.platform.apply_settings(
            &candidate.management_name,
            &candidate.management_key,
            &candidate.settings,
        );
        let failure = match outcome {
            Ok(details) if details.is_empty() => None,
            Ok(details) => Some((details.first().copied(), None)),
            Err(e) => {
                // A file that is not ours ends the process here; the respawn applies the
                // stored document, which undoes this change (or completes a restore).
                self.platform_failed("apply_settings", &e);
                Some((Some(HealthDetail::ManagementDown), Some(e.to_string())))
            }
        };
        if let Some((detail, error)) = failure {
            self.undo(id, UndoReason::HealthCheckFailed, detail, error);
            return;
        }
        if !needs_confirmation {
            {
                let mut inner = self.0.inner.lock().expect("app");
                if Self::is_change(&inner, id) {
                    Self::settle(&mut inner, id);
                }
            }
            self.follow_management_address();
            return;
        }
        // ENK-API-021: disconnect first, then wait; a request that arrives on a connection
        // accepted after this instant proves a new association.
        if let Err(e) = self.0.platform.disconnect_management_stations() {
            log::event(
                "platform_failed",
                &[("at", &"disconnect_management_stations"), ("error", &e)],
            );
        }
        let now = self.0.clock.now();
        let mut inner = self.0.inner.lock().expect("app");
        if let Some(c) = inner.change.as_mut().filter(|c| c.shape.id == id) {
            c.shape.state = ChangeState::WaitingConfirmation;
            c.disconnected_at = Some(now);
            c.confirm_deadline = Some(now + CONFIRMATION_LIMIT);
            log::event("change_waiting_confirmation", &[("id", &id)]);
        }
    }

    fn is_change(inner: &Inner, id: u64) -> bool {
        inner.change.as_ref().is_some_and(|c| c.shape.id == id)
    }

    /// The change becomes the stored document (ENK-CFG-010); the marker goes.
    fn settle(inner: &mut Inner, id: u64) {
        let Some(c) = inner.change.as_mut().filter(|c| c.shape.id == id) else {
            return;
        };
        let Some(candidate) = c.candidate.take() else {
            return;
        };
        c.shape.state = ChangeState::Settled;
        c.confirm_deadline = None;
        let kind = c.shape.kind;
        inner.store.data.owner = Some(candidate);
        inner.drift = false;
        inner.readings = None;
        if let Err(e) = inner.store.save() {
            log::event("store_failed", &[("at", &"settle"), ("error", &e)]);
        }
        if let Err(e) = inner.store.remove_pending() {
            log::event("store_failed", &[("at", &"remove_pending"), ("error", &e)]);
        }
        log::event(
            "change_settled",
            &[("id", &id), ("kind", &format!("{kind:?}"))],
        );
    }

    /// Restores the previous document on the device and reports the undone change
    /// (ENK-CFG-009). Runs on the caller's thread; the state stays `applying` meanwhile, which
    /// is what the device is doing.
    fn undo(
        &self,
        id: u64,
        reason: UndoReason,
        detail: Option<HealthDetail>,
        error: Option<String>,
    ) {
        let previous = {
            let mut inner = self.0.inner.lock().expect("app");
            let Some(c) = inner.change.as_mut().filter(|c| c.shape.id == id) else {
                return;
            };
            c.undoing = true;
            c.candidate = None;
            c.shape.state = ChangeState::Applying;
            c.confirm_deadline = None;
            c.previous.clone()
        };
        let restored = self.0.platform.apply_settings(
            &previous.management_name,
            &previous.management_key,
            &previous.settings,
        );
        if let Err(e) = restored {
            self.platform_failed("undo", &e);
        }
        let mut inner = self.0.inner.lock().expect("app");
        if let Some(c) = inner.change.as_mut().filter(|c| c.shape.id == id) {
            c.shape.state = ChangeState::Undone;
            c.shape.reason = Some(reason);
            c.shape.detail = detail;
            c.undoing = false;
        }
        inner.drift = false;
        inner.readings = None;
        if let Err(e) = inner.store.remove_pending() {
            log::event("store_failed", &[("at", &"remove_pending"), ("error", &e)]);
        }
        log::event(
            "change_undone",
            &[
                ("id", &id),
                ("reason", &format!("{reason:?}")),
                ("detail", &format!("{detail:?}")),
                ("error", &error.unwrap_or_default()),
            ],
        );
    }

    /// ENK-API-022: called for every request that carries a session header, before the
    /// request is handled, so that this very request already sees the change settled.
    pub fn note_owner_request(&self, token: &str, accepted: Instant) {
        let mut inner = self.0.inner.lock().expect("app");
        let now = self.0.clock.now();
        let id = match inner.change.as_ref() {
            Some(c)
                if c.shape.state == ChangeState::WaitingConfirmation
                    && !c.undoing
                    && c.disconnected_at.is_some_and(|t| accepted > t)
                    && c.confirm_deadline.is_some_and(|d| now < d) =>
            {
                c.shape.id
            }
            _ => return,
        };
        if inner.sessions.authenticate(token, now) == Some(SessionKind::Owner) {
            log::event("change_confirmed", &[("id", &id)]);
            Self::settle(&mut inner, id);
        }
    }

    /// Called from `tick`: a waiting change whose time ran out is undone (ENK-CFG-008).
    pub(super) fn tick_change(&self) {
        let expired = {
            let mut inner = self.0.inner.lock().expect("app");
            let now = self.0.clock.now();
            inner.change.as_mut().and_then(|c| {
                let expired = c.shape.state == ChangeState::WaitingConfirmation
                    && !c.undoing
                    && c.confirm_deadline.is_some_and(|d| now >= d);
                // Decided here, under the lock: no request can confirm it any more.
                if expired {
                    c.undoing = true;
                }
                expired.then_some(c.shape.id)
            })
        };
        if let Some(id) = expired {
            let app = self.clone();
            std::thread::Builder::new()
                .name(format!("enkrnettd-undo-{id}"))
                .spawn(move || app.undo(id, UndoReason::NotConfirmed, None, None))
                .expect("spawn undo");
        }
    }
}
