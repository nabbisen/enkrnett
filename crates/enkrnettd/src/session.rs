//! Sessions (ENK-AUTH-001 to 006; ENK-OWN-005): in memory only, identified by a random token,
//! at most four, ended after 15 minutes without a request or 12 hours in any case.

use std::time::{Duration, Instant};

use enkrnett_model::api::SessionKind;

pub const MAX_SESSIONS: usize = 4;
pub const IDLE_LIMIT: Duration = Duration::from_secs(15 * 60);
pub const LIFE_LIMIT: Duration = Duration::from_secs(12 * 60 * 60);
/// A claim session ends after 5 minutes without a request (ENK-OWN-005).
pub const CLAIM_IDLE_LIMIT: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Debug)]
struct Entry {
    token: String,
    kind: SessionKind,
    created: Instant,
    last_used: Instant,
}

impl Entry {
    fn idle_limit(&self) -> Duration {
        match self.kind {
            SessionKind::Claim => CLAIM_IDLE_LIMIT,
            SessionKind::Owner => IDLE_LIMIT,
        }
    }

    fn alive(&self, now: Instant) -> bool {
        now < self.created + LIFE_LIMIT && now < self.last_used + self.idle_limit()
    }
}

/// What a client learns when a session is created.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Created {
    pub token: String,
    pub idle_ends_in: u64,
    pub ends_in: u64,
}

#[derive(Default)]
pub struct SessionStore {
    entries: Vec<Entry>,
}

fn equal_constant_time(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

impl SessionStore {
    pub fn new() -> SessionStore {
        SessionStore::default()
    }

    fn expire(&mut self, now: Instant) {
        self.entries.retain(|e| e.alive(now));
    }

    /// Creates a session with a fresh token. When four exist, the one idle longest ends
    /// (ENK-AUTH-005).
    pub fn create(&mut self, kind: SessionKind, token: String, now: Instant) -> Created {
        self.expire(now);
        while self.entries.len() >= MAX_SESSIONS {
            let oldest = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(i, _)| i)
                .expect("non-empty");
            self.entries.remove(oldest);
        }
        let entry = Entry {
            token: token.clone(),
            kind,
            created: now,
            last_used: now,
        };
        let created = Created {
            token,
            idle_ends_in: entry.idle_limit().as_secs(),
            ends_in: LIFE_LIMIT.as_secs(),
        };
        self.entries.push(entry);
        created
    }

    /// Looks a token up, touching the session. `None` for an unknown or ended token.
    pub fn authenticate(&mut self, token: &str, now: Instant) -> Option<SessionKind> {
        self.expire(now);
        let entry = self
            .entries
            .iter_mut()
            .find(|e| equal_constant_time(&e.token, token))?;
        entry.last_used = now;
        Some(entry.kind)
    }

    /// Remaining `(idle_ends_in, ends_in)` of a session, without touching it.
    pub fn remaining(&self, token: &str, now: Instant) -> Option<(u64, u64)> {
        let e = self
            .entries
            .iter()
            .find(|e| equal_constant_time(&e.token, token))?;
        if !e.alive(now) {
            return None;
        }
        Some((
            crate::clock::seconds_until(now, e.last_used + e.idle_limit()),
            crate::clock::seconds_until(now, e.created + LIFE_LIMIT),
        ))
    }

    pub fn end(&mut self, token: &str) {
        self.entries
            .retain(|e| !equal_constant_time(&e.token, token));
    }

    pub fn end_all(&mut self) {
        self.entries.clear();
    }

    pub fn end_kind(&mut self, kind: SessionKind) {
        self.entries.retain(|e| e.kind != kind);
    }

    /// Whether a live session of `kind` exists.
    pub fn has_kind(&mut self, kind: SessionKind, now: Instant) -> bool {
        self.expire(now);
        self.entries.iter().any(|e| e.kind == kind)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests;
