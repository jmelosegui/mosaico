//! Reading the settings from `config.toml`, and writing the edits back.

use mosaico_core::Error;
use mosaico_core::config::ConfigStore;

use super::Settings;

/// The settings found in the config folder, and why they cannot be
/// edited, when they cannot.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The settings in effect. The defaults when the file cannot be
    /// read.
    pub settings: Settings,
    /// Why `config.toml` cannot be edited here: it cannot be read, or the
    /// daemon would not load it. Saving over it could lose what the user
    /// wrote, so the window waits for it to be fixed.
    pub problem: Option<String>,
}

impl Loaded {
    /// The settings of `config.toml` in `store`, with whether mosaico
    /// starts at sign-in. A missing file has the defaults and can be
    /// edited; the first save creates it.
    pub fn read(store: &ConfigStore, start_at_sign_in: bool) -> Self {
        let read = store.open_settings().and_then(|doc| {
            doc.check()?;
            Ok(Settings::read(&doc, start_at_sign_in))
        });
        match read {
            Ok(settings) => Self {
                settings,
                problem: None,
            },
            Err(e) => Self::blocked(reason(e), start_at_sign_in),
        }
    }

    /// The defaults, which cannot be edited because of `problem`.
    pub fn blocked(problem: String, start_at_sign_in: bool) -> Self {
        let mut settings = Settings::defaults();
        settings.start_at_sign_in = start_at_sign_in;
        Self {
            settings,
            problem: Some(problem),
        }
    }
}

/// Writes the settings of `config.toml` that `edit` changes from `saved`
/// to the file in `store`, leaving the rest of it as it is. A setting
/// changed back to its default is removed from the file.
///
/// # Errors
///
/// Says why the file was not written: it cannot be read or written, or
/// would not load with the edits. It is then left as it was.
pub fn save(store: &ConfigStore, saved: &Settings, edit: &Settings) -> Result<(), String> {
    let mut doc = store.open_settings().map_err(reason)?;
    for key in edit.changed_keys(saved) {
        doc.put(key, edit.get(key)).map_err(reason)?;
    }
    if doc.is_dirty() {
        doc.save().map_err(reason)?;
    }
    Ok(())
}

/// What is wrong, without the file's path, which the window shows apart.
fn reason(error: Error) -> String {
    match error {
        Error::Config { message, .. } => message,
        other => other.to_string(),
    }
}
