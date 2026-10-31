//! Daemon configuration. Values come from the command line (and later from the device profile);
//! nothing here is operator-facing.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

/// Everything the daemon needs to know before it starts.
#[derive(Clone, Debug)]
pub struct Config {
    /// The address the HTTP listener binds (ENK-CFG-025: the management address, never all).
    pub listen: SocketAddr,
    /// Where the settings and the owner record are kept.
    pub state_dir: PathBuf,
    /// The UI's static files; `None` serves a built-in placeholder page.
    pub ui_dir: Option<PathBuf>,
    /// A development build identifies itself in the greeting (ENK-HARD-007).
    pub development: bool,
    /// How long the daemon waits after answering before it acts on confirm, restart, and erase
    /// (ENK-API-015: at least 2 s in a release).
    pub act_delay: Duration,
    /// Host names the `Host` header may carry besides the listen address (ENK-SEC-009).
    pub hosts: Vec<String>,
    /// How long a client has to deliver a complete request (ENK-SEC-018: 10 s).
    pub request_timeout: Duration,
    /// How long a response may take (ENK-SEC-018: 30 s).
    pub response_timeout: Duration,
    /// The Enkrnett version shown to the operator.
    pub version: String,
    /// Languages the UI ships, as the greeting lists them.
    pub languages: Vec<String>,
    /// argon2id parameters `(memory KiB, iterations, parallelism)` (ENK-FOOT-017).
    pub argon2: (u32, u32, u32),
}

impl Default for Config {
    fn default() -> Config {
        Config {
            listen: "127.0.0.1:8080".parse().expect("literal"),
            state_dir: PathBuf::from("/var/lib/enkrnett"),
            ui_dir: None,
            development: false,
            act_delay: Duration::from_secs(2),
            hosts: Vec::new(),
            request_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(30),
            version: env!("CARGO_PKG_VERSION").to_string(),
            languages: vec!["en".to_string(), "ja".to_string()],
            argon2: (8 * 1024, 3, 1),
        }
    }
}

impl Config {
    /// Whether a `Host` header value names this device (ENK-SEC-009). A port is ignored; an
    /// IPv6 literal in brackets is accepted for the listen address.
    pub fn allowed_host(&self, host: &str) -> bool {
        let host = host.trim();
        let name = if let Some(rest) = host.strip_prefix('[') {
            rest.split(']').next().unwrap_or("")
        } else {
            host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host)
        };
        if name.is_empty() {
            return false;
        }
        if name
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip == self.listen.ip())
        {
            return true;
        }
        self.hosts.iter().any(|h| h.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_matching() {
        let mut c = Config {
            listen: "192.168.207.1:80".parse().unwrap(),
            hosts: vec!["enkrnett.home.arpa".into()],
            ..Config::default()
        };
        assert!(c.allowed_host("192.168.207.1"));
        assert!(c.allowed_host("192.168.207.1:80"));
        assert!(c.allowed_host("ENKRNETT.home.arpa"));
        assert!(!c.allowed_host("192.168.207.2"));
        assert!(!c.allowed_host("evil.example"));
        assert!(!c.allowed_host(""));
        c.listen = "[::1]:80".parse().unwrap();
        assert!(c.allowed_host("[::1]:8080"));
    }
}
