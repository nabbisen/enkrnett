//! The release index (chapter 11 §2, ENK-RLS-006 as amended by RFC 016 §9) and release
//! versions. The index is one small JSON document, signed with signify (ENK-RLS-004); its
//! signature is verified before this module parses it, so what is checked here is shape and
//! plausibility, not trust. Unknown fields are accepted: a later tooling may add some without
//! leaving older devices unable to learn about releases.
//!
//! ```json
//! { "format": 1, "sequence": 7,
//!   "releases": [ { "profile": "elecom_wrc-1167gst2", "version": "0.3.0",
//!                   "baseline": "25.12", "from_baseline": "25.12",
//!                   "file": "enkrnett-0.3.0-25.12-elecom_wrc-1167gst2-sysupgrade.bin",
//!                   "size": 6291456, "sha512": "…", "min_free_kib": 24576,
//!                   "notes_url": "https://…/0.3.0", "install_seconds": 180 } ] }
//! ```

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::api::Available;

pub const INDEX_FORMAT: u32 = 1;
/// The index is small by requirement; anything larger is refused before parsing.
pub const INDEX_MAX_BYTES: usize = 64 * 1024;
/// How many releases one index may list.
pub const INDEX_MAX_RELEASES: usize = 64;
/// An image larger than this is not for the devices of chapter 5.
pub const IMAGE_MAX_BYTES: u64 = 256 * 1024 * 1024;

/// A release version `major.minor.patch` (ENK-RLS-001: semantic versioning, no prefix).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        let mut parts = text.split('.');
        let mut next = || {
            let p = parts.next()?;
            if p.is_empty() || p.len() > 6 || !p.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            if p.len() > 1 && p.starts_with('0') {
                return None;
            }
            p.parse::<u32>().ok()
        };
        let v = Version {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(v)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseIndex {
    pub format: u32,
    /// Increases with every publication (ENK-UPD-009).
    pub sequence: u64,
    pub releases: Vec<ReleaseEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEntry {
    /// The device profile identifier (`openwrt/profiles/<id>`).
    pub profile: String,
    pub version: String,
    /// The OpenWrt baseline the image is built on.
    pub baseline: String,
    /// The baseline an installed device must run to take this image: the same, or the one
    /// before when this release is an upgrade (ENK-UPD-003, ENK-UPD-007).
    pub from_baseline: String,
    /// `enkrnett-<version>-<baseline>-<profile>-<kind>.bin` (ENK-RLS-002).
    pub file: String,
    pub size: u64,
    /// Lower-case hex SHA-512 of the image file.
    pub sha512: String,
    /// Free memory the installation needs, beyond the image itself (ENK-UPD-003).
    pub min_free_kib: u64,
    pub notes_url: String,
    /// What the confirmation screen tells the operator (RFC 017 §5.6).
    pub install_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexError {
    /// Not JSON of the expected shape.
    Json,
    /// `format` is not one this release reads.
    Format(u32),
    /// A field of one release entry is missing or implausible.
    Field {
        entry: usize,
        field: &'static str,
    },
    TooManyReleases,
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::Json => write!(f, "index: not the expected JSON"),
            IndexError::Format(v) => {
                write!(f, "index: format {v}, this release reads {INDEX_FORMAT}")
            }
            IndexError::Field { entry, field } => write!(f, "index: release {entry}: bad {field}"),
            IndexError::TooManyReleases => {
                write!(f, "index: more than {INDEX_MAX_RELEASES} releases")
            }
        }
    }
}

fn plain(text: &str, max: usize, extra: &[u8]) -> bool {
    !text.is_empty()
        && text.len() <= max
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(&b))
}

impl ReleaseEntry {
    fn check(&self, entry: usize) -> Result<(), IndexError> {
        let bad = |field| IndexError::Field { entry, field };
        if !plain(&self.profile, 64, b"_-") {
            return Err(bad("profile"));
        }
        let version = Version::parse(&self.version).ok_or(bad("version"))?;
        if !plain(&self.baseline, 16, b".") || !plain(&self.from_baseline, 16, b".") {
            return Err(bad("baseline"));
        }
        let prefix = format!("enkrnett-{version}-{}-{}-", self.baseline, self.profile);
        if !plain(&self.file, 128, b"._-")
            || !self.file.starts_with(&prefix)
            || !self.file.ends_with(".bin")
            || self.file.len() <= prefix.len() + 4
        {
            return Err(bad("file"));
        }
        if self.size == 0 || self.size > IMAGE_MAX_BYTES {
            return Err(bad("size"));
        }
        if self.sha512.len() != 128
            || !self
                .sha512
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(bad("sha512"));
        }
        if self.min_free_kib > 1 << 20 {
            return Err(bad("min_free_kib"));
        }
        if self.notes_url.len() > 256
            || !self.notes_url.starts_with("https://")
            || self.notes_url.bytes().any(|b| !b.is_ascii_graphic())
        {
            return Err(bad("notes_url"));
        }
        if self.install_seconds == 0 || self.install_seconds > 3600 {
            return Err(bad("install_seconds"));
        }
        Ok(())
    }

    pub fn version(&self) -> Version {
        Version::parse(&self.version).expect("checked when parsed")
    }

    pub fn changes_baseline(&self) -> bool {
        self.baseline != self.from_baseline
    }

    /// The `available` shape of chapter 9 §4.
    pub fn available(&self) -> Available {
        Available {
            version: self.version.clone(),
            baseline: self.baseline.clone(),
            changes_baseline: self.changes_baseline(),
            size_kib: self.size.div_ceil(1024),
            notes_url: self.notes_url.clone(),
            install_seconds: self.install_seconds,
        }
    }
}

impl ReleaseIndex {
    /// Parses and checks an index whose signature has already been verified.
    pub fn parse(bytes: &[u8]) -> Result<ReleaseIndex, IndexError> {
        if bytes.len() > INDEX_MAX_BYTES {
            return Err(IndexError::Json);
        }
        let index: ReleaseIndex = serde_json::from_slice(bytes).map_err(|_| IndexError::Json)?;
        if index.format != INDEX_FORMAT {
            return Err(IndexError::Format(index.format));
        }
        if index.releases.len() > INDEX_MAX_RELEASES {
            return Err(IndexError::TooManyReleases);
        }
        for (i, r) in index.releases.iter().enumerate() {
            r.check(i)?;
        }
        Ok(index)
    }

    /// The release to offer a device: the highest version listed for its profile that follows
    /// its baseline and is newer than what it runs (ENK-UPD-009: never the installed or an
    /// older one). `None` is "up to date".
    pub fn offer(
        &self,
        profile: &str,
        installed: Version,
        baseline: &str,
    ) -> Option<&ReleaseEntry> {
        self.releases
            .iter()
            .filter(|r| r.profile == profile && r.from_baseline == baseline)
            .filter(|r| r.version() > installed)
            .max_by(|a, b| {
                a.version()
                    .cmp(&b.version())
                    // Of two equal versions prefer the one that stays on the baseline.
                    .then_with(|| match (a.changes_baseline(), b.changes_baseline()) {
                        (false, true) => Ordering::Greater,
                        (true, false) => Ordering::Less,
                        _ => Ordering::Equal,
                    })
            })
    }
}

#[cfg(test)]
mod tests;
