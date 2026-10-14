//! Keeps the record of windows hidden by workspace switching on disk.
//!
//! A daemon that is killed or crashes leaves those windows hidden. The
//! record lets the next daemon show them again before it starts tiling.
//! It lives in the instance's data directory, next to the PID file.

use std::fs;
use std::path::PathBuf;

use crate::Instance;

use super::HiddenWindows;

/// File name of the record inside the instance's data directory.
const FILE_NAME: &str = "hidden-windows.json";

/// Where the record of hidden windows is kept.
pub struct HiddenFile {
    path: PathBuf,
}

impl HiddenFile {
    /// The record at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The record of `instance`, in its data directory.
    pub fn for_instance(instance: &Instance) -> Self {
        Self::new(instance.data_dir().join(FILE_NAME))
    }

    /// Reads the record a previous daemon left. A missing file is an
    /// empty record; an unreadable one is logged and treated as empty,
    /// since there is nothing better to do with it.
    pub fn load(&self) -> HiddenWindows {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(_) => return HiddenWindows::default(),
        };
        HiddenWindows::from_text(&text).unwrap_or_else(|e| {
            crate::log_warn!("ignoring {}: {e}", self.path.display());
            HiddenWindows::default()
        })
    }

    /// Writes the record, or deletes the file when nothing is hidden.
    ///
    /// Written to a temporary file and renamed over the old one, so a
    /// daemon killed mid-write leaves the previous record rather than
    /// half of a new one.
    pub fn save(&self, record: &HiddenWindows) {
        if record.is_empty() {
            let _ = fs::remove_file(&self.path);
            return;
        }
        let result = (|| {
            if let Some(dir) = self.path.parent() {
                fs::create_dir_all(dir)?;
            }
            let temp = self.path.with_extension("json.tmp");
            fs::write(&temp, record.to_text())?;
            fs::rename(&temp, &self.path)
        })();
        if let Err(e) = result {
            crate::log_warn!("could not write {}: {e}", self.path.display());
        }
    }
}

#[cfg(test)]
#[path = "tests/hidden_file_tests.rs"]
mod tests;
