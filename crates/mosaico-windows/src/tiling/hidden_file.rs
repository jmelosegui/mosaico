//! Keeps the record of windows hidden by workspace switching on disk.
//!
//! A daemon that is killed or crashes leaves those windows hidden. The
//! record lets the next daemon show them again before it starts tiling.
//! It lives in the instance's data directory, next to the PID file.

use std::fs;
use std::path::PathBuf;

use mosaico_core::Instance;
use mosaico_core::tiler::HiddenWindows;

/// File name of the record inside the instance's data directory.
const FILE_NAME: &str = "hidden-windows.json";

/// Where the record of hidden windows is kept.
pub(super) struct HiddenFile {
    path: PathBuf,
}

impl HiddenFile {
    /// The record at `path`.
    pub(super) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The record of this process's instance, if it has a data
    /// directory.
    pub(super) fn current() -> Option<Self> {
        Instance::current()
            .ok()
            .map(|instance| Self::new(instance.data_dir().join(FILE_NAME)))
    }

    /// Reads the record a previous daemon left. A missing file is an
    /// empty record; an unreadable one is logged and treated as empty,
    /// since there is nothing better to do with it.
    pub(super) fn load(&self) -> HiddenWindows {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(_) => return HiddenWindows::default(),
        };
        HiddenWindows::from_text(&text).unwrap_or_else(|e| {
            mosaico_core::log_warn!("ignoring {}: {e}", self.path.display());
            HiddenWindows::default()
        })
    }

    /// Writes the record, or deletes the file when nothing is hidden.
    ///
    /// Written to a temporary file and renamed over the old one, so a
    /// daemon killed mid-write leaves the previous record rather than
    /// half of a new one.
    pub(super) fn save(&self, record: &HiddenWindows) {
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
            mosaico_core::log_warn!("could not write {}: {e}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use mosaico_core::WindowId;
    use mosaico_core::config::HidingBehaviour;
    use mosaico_core::tiler::{Effect, WindowIdentity};

    use super::*;

    fn temp_file(test: &str) -> HiddenFile {
        let dir = std::env::temp_dir()
            .join("mosaico-hidden-file-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        HiddenFile::new(dir.join(FILE_NAME))
    }

    fn one_hidden() -> HiddenWindows {
        let mut record = HiddenWindows::default();
        record.applied(
            &Effect::Hide {
                window: WindowId::from_raw(0x4014E),
                how: HidingBehaviour::Cloak,
            },
            |_| {
                Some(WindowIdentity {
                    process: 7,
                    class: "App".into(),
                })
            },
        );
        record
    }

    #[test]
    fn a_missing_file_is_an_empty_record() {
        // Arrange
        let file = temp_file("missing");

        // Act / Assert
        assert!(file.load().is_empty());
    }

    #[test]
    fn a_saved_record_loads_back() {
        // Arrange
        let file = temp_file("round-trip");
        let record = one_hidden();

        // Act
        file.save(&record);

        // Assert
        assert_eq!(file.load(), record);
    }

    #[test]
    fn saving_an_empty_record_removes_the_file() {
        // Arrange
        let file = temp_file("empty");
        file.save(&one_hidden());

        // Act
        file.save(&HiddenWindows::default());

        // Assert
        assert!(!file.path.exists());
    }

    #[test]
    fn a_corrupt_file_is_an_empty_record() {
        // Arrange
        let file = temp_file("corrupt");
        fs::create_dir_all(file.path.parent().unwrap()).unwrap();
        fs::write(&file.path, "{ this is not a record").unwrap();

        // Act / Assert
        assert!(file.load().is_empty());
    }
}
