//! What the daemon keeps on flash: the device identifier, the owner record, the settings. One
//! JSON file with a format version (ENK-BASE-005), written to a temporary name and renamed into
//! place (ENK-CFG-006). Erase removes it (ENK-OWN-018).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use enkrnett_model::STORAGE_FORMAT;
use enkrnett_model::settings::Settings;
use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "state.json";

/// The owner and everything the claim created.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerRecord {
    pub password_hash: String,
    pub management_name: String,
    pub management_key: String,
    pub settings: Settings,
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
        let path = self.path();
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        let bytes = serde_json::to_vec_pretty(&self.data).map_err(io::Error::other)?;
        {
            let mut f = fs::File::create(&tmp)?;
            io::Write::write_all(&mut f, &bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)?;
        if let Ok(d) = fs::File::open(&self.dir) {
            let _ = d.sync_all();
        }
        Ok(())
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
