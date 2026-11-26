//! Monotonic time (ENK-AUTH-004, ENK-CFG-012): every duration the daemon measures comes from a
//! clock that does not depend on the date being set. Tests use [`MockClock`].

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// `Instant::now()`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock that moves only when told to.
#[derive(Clone)]
pub struct MockClock(Arc<Mutex<Instant>>);

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockClock {
    pub fn new() -> MockClock {
        MockClock(Arc::new(Mutex::new(Instant::now())))
    }

    pub fn advance(&self, by: Duration) {
        *self.0.lock().expect("clock") += by;
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        *self.0.lock().expect("clock")
    }
}

/// Whole seconds from `now` to `deadline`, zero when the deadline has passed.
/// Rounded up, so that a client which waits exactly this long is not refused again.
pub fn seconds_until(now: Instant, deadline: Instant) -> u64 {
    let d = deadline.saturating_duration_since(now);
    d.as_secs() + u64::from(d.subsec_nanos() > 0)
}

/// Whole seconds from `then` to `now`.
pub fn seconds_since(then: Instant, now: Instant) -> u64 {
    now.saturating_duration_since(then).as_secs()
}
