//! Owner password hashing (ENK-OWN-011, ENK-FOOT-017) and the login delay (ENK-OWN-012).

use std::time::{Duration, Instant};

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use rand_core::OsRng;

/// argon2id with the parameters of the device profile.
pub struct Hasher {
    argon: Argon2<'static>,
}

impl Hasher {
    /// `memory_kib`, `iterations`, `parallelism` as ENK-FOOT-017 records them per profile.
    pub fn new(memory_kib: u32, iterations: u32, parallelism: u32) -> Hasher {
        let params =
            Params::new(memory_kib, iterations, parallelism, None).expect("argon2 parameters");
        Hasher {
            argon: Argon2::new(Algorithm::Argon2id, Version::V0x13, params),
        }
    }

    /// A salted hash in PHC string form. The password itself is never stored.
    pub fn hash(&self, password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        self.argon
            .hash_password(password.as_bytes(), &salt)
            .expect("argon2 hash")
            .to_string()
    }

    /// Whether `password` produced `hash`. A malformed hash verifies as false.
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed) => self
                .argon
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    }
}

/// After 5 consecutive failed logins the device refuses further attempts for 30 seconds,
/// doubling after each further failure, up to 15 minutes. A successful login or a restart ends
/// the delay (ENK-OWN-012).
#[derive(Debug, Default)]
pub struct LoginGuard {
    consecutive_failures: u32,
    refuse_until: Option<Instant>,
}

impl LoginGuard {
    pub const THRESHOLD: u32 = 5;
    pub const FIRST_DELAY: Duration = Duration::from_secs(30);
    pub const MAX_DELAY: Duration = Duration::from_secs(15 * 60);

    /// `Ok` when a login may be attempted now; otherwise the seconds to wait (`retry_in`).
    pub fn check(&self, now: Instant) -> Result<(), u64> {
        match self.refuse_until {
            Some(until) if until > now => Err(crate::clock::seconds_until(now, until).max(1)),
            _ => Ok(()),
        }
    }

    /// Records a failed login.
    pub fn failure(&mut self, now: Instant) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.consecutive_failures >= Self::THRESHOLD {
            let doublings = self.consecutive_failures - Self::THRESHOLD;
            let delay = Self::FIRST_DELAY
                .checked_mul(1u32 << doublings.min(10))
                .unwrap_or(Self::MAX_DELAY)
                .min(Self::MAX_DELAY);
            self.refuse_until = Some(now + delay);
        }
    }

    /// Records a successful login: the delay ends.
    pub fn success(&mut self) {
        self.consecutive_failures = 0;
        self.refuse_until = None;
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}

#[cfg(test)]
mod tests;
