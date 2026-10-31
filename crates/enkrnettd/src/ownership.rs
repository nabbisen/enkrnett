//! The ownership state machine (chapter 4 §1, §2): the only states and the only transitions,
//! with the setup window, its one extension, and the pending claim's 10 minutes. Pure apart
//! from the `Instant`s it is given; the application turns its events into platform calls.

use std::time::{Duration, Instant};

use enkrnett_model::state::OwnershipState;

use crate::store::OwnerRecord;

/// The setup window stays open for 30 minutes (ENK-OWN-002).
pub const SETUP_WINDOW: Duration = Duration::from_secs(30 * 60);
/// It is extended once by 15 minutes when a claim is under way at its end (ENK-OWN-002).
pub const SETUP_EXTENSION: Duration = Duration::from_secs(15 * 60);
/// A confirmed claim becomes final within 10 minutes or is forgotten (ENK-OWN-008).
pub const PENDING_LIMIT: Duration = Duration::from_secs(10 * 60);

/// A claim that was confirmed but is not final.
#[derive(Clone, Debug)]
pub struct PendingClaim {
    pub record: OwnerRecord,
    pub since: Instant,
}

/// What a transition asks the application to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Offer the setup network.
    SetupOpened,
    /// The window closed: offer nothing.
    SetupClosed,
    /// The pending claim was not made final in time: forget it, offer the setup network again.
    PendingExpired,
}

#[derive(Debug)]
pub struct Ownership {
    state: OwnershipState,
    window_end: Option<Instant>,
    extended: bool,
    returned_from_pending: bool,
    pending: Option<PendingClaim>,
}

impl Ownership {
    /// The state at start: `CLAIMED` with an owner, otherwise `SETUP` with a fresh window
    /// (ENK-OWN-001). A device that restarts without an owner always starts here, whatever its
    /// state was before.
    pub fn start(has_owner: bool, now: Instant) -> (Ownership, Option<Event>) {
        if has_owner {
            (
                Ownership {
                    state: OwnershipState::Claimed,
                    window_end: None,
                    extended: false,
                    returned_from_pending: false,
                    pending: None,
                },
                None,
            )
        } else {
            (
                Ownership {
                    state: OwnershipState::Setup,
                    window_end: Some(now + SETUP_WINDOW),
                    extended: false,
                    returned_from_pending: false,
                    pending: None,
                },
                Some(Event::SetupOpened),
            )
        }
    }

    pub fn state(&self) -> OwnershipState {
        self.state
    }

    /// Seconds until the setup window closes; zero outside `SETUP`.
    pub fn window_ends_in(&self, now: Instant) -> u64 {
        match (self.state, self.window_end) {
            (OwnershipState::Setup, Some(end)) => crate::clock::seconds_until(now, end),
            _ => 0,
        }
    }

    /// Seconds until a pending claim is forgotten; `None` outside `PENDING`.
    pub fn final_in(&self, now: Instant) -> Option<u64> {
        self.pending
            .as_ref()
            .map(|p| crate::clock::seconds_until(now, p.since + PENDING_LIMIT))
    }

    pub fn pending(&self) -> Option<&PendingClaim> {
        self.pending.as_ref()
    }

    /// Advances the timers. `claim_under_way` says whether a claim session exists, which earns
    /// the window its one extension.
    pub fn tick(&mut self, now: Instant, claim_under_way: bool) -> Vec<Event> {
        let mut events = Vec::new();
        match self.state {
            OwnershipState::Setup => {
                if let Some(end) = self.window_end {
                    if now >= end {
                        if !self.extended && (claim_under_way || self.returned_from_pending) {
                            self.extended = true;
                            self.window_end = Some(end + SETUP_EXTENSION);
                        } else {
                            self.state = OwnershipState::Unclaimed;
                            self.window_end = None;
                            events.push(Event::SetupClosed);
                        }
                    }
                }
            }
            OwnershipState::Pending => {
                if let Some(p) = &self.pending {
                    if now >= p.since + PENDING_LIMIT {
                        let spent = now - p.since;
                        self.pending = None;
                        self.state = OwnershipState::Setup;
                        self.returned_from_pending = true;
                        // Time spent in PENDING does not count against the window.
                        self.window_end = self.window_end.map(|e| e + spent);
                        events.push(Event::PendingExpired);
                    }
                }
            }
            OwnershipState::Unclaimed | OwnershipState::Claimed => {}
        }
        events
    }

    /// The operator confirmed setup (step 7): `SETUP` to `PENDING`.
    pub fn confirm(&mut self, record: OwnerRecord, now: Instant) -> Result<(), OwnershipState> {
        if self.state != OwnershipState::Setup {
            return Err(self.state);
        }
        self.pending = Some(PendingClaim { record, since: now });
        self.state = OwnershipState::Pending;
        Ok(())
    }

    /// The owner logged in through the management network (step 10): `PENDING` to `CLAIMED`.
    /// Returns the record to persist.
    pub fn make_final(&mut self) -> Option<OwnerRecord> {
        if self.state != OwnershipState::Pending {
            return None;
        }
        let record = self.pending.take()?.record;
        self.state = OwnershipState::Claimed;
        self.window_end = None;
        Some(record)
    }

    /// Erase: `CLAIMED` (or anything) to `UNCLAIMED`. The device then restarts.
    pub fn erase(&mut self) {
        self.state = OwnershipState::Unclaimed;
        self.pending = None;
        self.window_end = None;
    }
}

#[cfg(test)]
mod tests;
