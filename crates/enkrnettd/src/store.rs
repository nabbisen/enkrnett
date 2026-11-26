//! What the daemon keeps on flash: the device identifier, the owner record, the settings, and,
//! while a change runs, the pending marker. JSON files with a format version (ENK-BASE-005),
//! each written to a temporary name and renamed into place (ENK-CFG-006). Erase removes them
//! (ENK-OWN-018).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use enkrnett_model::STORAGE_FORMAT;
use enkrnett_model::api::ChangeKind;
use enkrnett_model::settings::Settings;
use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "state.json";
pub const PENDING_FILE_NAME: &str = "pending.json";

/// The owner and everything the claim created.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerRecord {
    pub password_hash: String,
    pub management_name: String,
    pub management_key: String,
    pub settings: Settings,
}

/// The pending marker of chapter 6 §4: on flash from before the owned sections are written until
/// the change settles or is undone. `previous` is the snapshot: under ENK-CFG-001 the owned
/// sections are a pure function of the desired document, so the previous document restores them
/// exactly. A marker found at start-up means the change is undone (ENK-CFG-009).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingMarker {
    pub format: u32,
    pub id: u64,
    pub kind: ChangeKind,
    pub from_version: u64,
    pub to_version: u64,
    pub needs_confirmation: bool,
    pub previous: OwnerRecord,
}

/// The whole stored document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stored {
    pub format: u32,
    pub device_id: String,
    pub owner: Option<OwnerRecord>,
    /// Highest release-index sequence accepted so far (ENK-UPD-009).
    #[serde(default)]
    pub index_sequence: u64,
}

impl Stored {
    fn fresh(device_id: String) -> Stored {
        Stored {
            format: STORAGE_FORMAT,
            device_id,
            owner: None,
            index_sequence: 0,
        }
    }
}

#[derive(Debug)]
pub struct Store {
    dir: PathBuf,
    pub data: Stored,
}

impl Store {
    /// Opens the state directory, creating it and a fresh document when none exists. A document
    /// written by a newer format is refused rather than misread.
    pub fn open(dir: &Path, new_device_id: impl Fn() -> String) -> io::Result<Store> {
        fs::create_dir_all(dir)?;
        let path = dir.join(FILE_NAME);
        let data = match fs::read(&path) {
            Ok(bytes) => {
                let stored: Stored = serde_json::from_slice(&bytes).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{}: {e}", path.display()),
                    )
                })?;
                if stored.format > STORAGE_FORMAT {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "{} has format {}, this daemon reads up to {}",
                            path.display(),
                            stored.format,
                            STORAGE_FORMAT
                        ),
                    ));
                }
                stored
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let fresh = Stored::fresh(new_device_id());
                let store = Store {
                    dir: dir.to_path_buf(),
                    data: fresh,
                };
                store.save()?;
                return Ok(store);
            }
            Err(e) => return Err(e),
        };
        Ok(Store {
            dir: dir.to_path_buf(),
            data,
        })
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Writes the document atomically.
    pub fn save(&self) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.data).map_err(io::Error::other)?;
        self.write_atomically(FILE_NAME, &bytes)
    }

    /// Writes the pending marker (ENK-CFG-006), before anything else of the change touches
    /// the device.
    pub fn write_pending(&self, marker: &PendingMarker) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(marker).map_err(io::Error::other)?;
        self.write_atomically(PENDING_FILE_NAME, &bytes)
    }

    /// The marker left by a change that did not finish, if any. An unreadable marker counts
    /// as one: the change is undone either way, and the file goes.
    pub fn read_pending(&self) -> Option<Result<PendingMarker, String>> {
        match fs::read(self.dir.join(PENDING_FILE_NAME)) {
            Ok(bytes) => Some(serde_json::from_slice(&bytes).map_err(|e| e.to_string())),
            Err(_) => None,
        }
    }

    /// Removes the marker: the change settled or was undone.
    pub fn remove_pending(&self) -> io::Result<()> {
        match fs::remove_file(self.dir.join(PENDING_FILE_NAME)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => {
                self.sync_dir();
                Ok(())
            }
        }
    }

    pub fn has_pending(&self) -> bool {
        self.dir.join(PENDING_FILE_NAME).exists()
    }

    fn write_atomically(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let path = self.dir.join(name);
        let tmp = self.dir.join(format!("{name}.tmp"));
        write_private(&tmp, bytes)?;
        fs::rename(&tmp, &path)?;
        self.sync_dir();
        Ok(())
    }

    fn sync_dir(&self) {
        if let Ok(d) = fs::File::open(&self.dir) {
            let _ = d.sync_all();
        }
    }

    /// Erase (ENK-OWN-018): removes every file of the state directory and starts a fresh
    /// document with a new identifier.
    pub fn erase(&mut self, new_device_id: impl Fn() -> String) -> io::Result<()> {
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            }
        }
        self.data = Stored::fresh(new_device_id());
        self.save()
    }
}

#[cfg(test)]
mod tests;

/// Creates or truncates `path` with mode 0600, writes `bytes`, and syncs. The owner record holds
/// the password hash and the management key; nothing on the device but the daemon reads them.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    io::Write::write_all(&mut f, bytes)?;
    f.sync_all()?;
    // An existing file keeps its old mode on open; make sure.
    fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    Ok(())
}
