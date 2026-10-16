//! What one daemon hid that the next one has to show again.
//!
//! A workspace switch hides windows, and only a clean shutdown shows
//! them again. A daemon that is killed or crashes leaves them hidden,
//! and the next one cannot tell them apart from windows the system
//! keeps out of view for its own reasons, so it never adopts them.
//!
//! [`HiddenWindows`] is the record a platform keeps of those windows
//! while it applies effects, and persists, so that the next daemon can
//! show them again before it starts tiling. Window ids may be reused
//! once a window is gone, so each entry also remembers who owned the
//! window, and a window that is no longer the same one is left alone.

use std::collections::BTreeMap;
use std::io;

use serde::{Deserialize, Serialize};

use crate::config::HidingBehaviour;
use crate::{Error, Result, WindowId};

use super::Effect;

/// Version of the text written by [`HiddenWindows::to_text`].
const FORMAT_VERSION: u32 = 1;

/// What identifies a window beyond its id, which the platform may reuse
/// once the window is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowIdentity {
    /// Process that owns the window.
    pub process: u32,
    /// Window class or the platform's nearest equivalent.
    pub class: String,
}

/// Windows hidden by workspace switching, as a daemon leaves them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HiddenWindows {
    entries: BTreeMap<WindowId, (HidingBehaviour, WindowIdentity)>,
}

impl HiddenWindows {
    /// Updates the record for an effect the platform just applied.
    ///
    /// A hide adds the window, a show or a reveal removes it. `identity`
    /// looks up the window the effect is about, and is only called for a
    /// hide; a window it does not find is not recorded. Returns whether
    /// the record changed.
    pub fn applied(
        &mut self,
        effect: &Effect,
        identity: impl FnOnce(WindowId) -> Option<WindowIdentity>,
    ) -> bool {
        match *effect {
            Effect::Hide { window, how } => match identity(window) {
                Some(who) => self.entries.insert(window, (how, who.clone())) != Some((how, who)),
                None => self.entries.remove(&window).is_some(),
            },
            Effect::Show { window, .. } | Effect::Reveal { window } => self.forget(window),
            _ => false,
        }
    }

    /// Drops a window that no longer exists. Returns whether the record
    /// changed.
    pub fn forget(&mut self, window: WindowId) -> bool {
        self.entries.remove(&window).is_some()
    }

    /// Whether nothing is recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The effects that show again what a previous daemon hid.
    ///
    /// Each window is shown the way it was hidden. A window `identity`
    /// does not find, or finds owned by another process or of another
    /// class, is skipped: its id now belongs to a different window.
    pub fn restore(&self, identity: impl Fn(WindowId) -> Option<WindowIdentity>) -> Vec<Effect> {
        self.entries
            .iter()
            .filter(|(window, (_, who))| identity(**window).as_ref() == Some(who))
            .map(|(&window, &(how, _))| Effect::Show { window, how })
            .collect()
    }

    /// The record as text, to keep it across daemon runs.
    pub fn to_text(&self) -> String {
        let file = FileFormat {
            version: FORMAT_VERSION,
            hidden: self
                .entries
                .iter()
                .map(|(&window, (how, who))| Entry {
                    window,
                    how: *how,
                    process: who.process,
                    class: who.class.clone(),
                })
                .collect(),
        };
        serde_json::to_string_pretty(&file).unwrap_or_default()
    }

    /// Reads a record written by [`HiddenWindows::to_text`]. Text that is
    /// not such a record is an [`io::ErrorKind::InvalidData`] error.
    pub fn from_text(text: &str) -> Result<Self> {
        let file: FileFormat = serde_json::from_str(text)
            .map_err(|e| invalid_record(format!("unreadable record: {e}")))?;
        if file.version != FORMAT_VERSION {
            return Err(invalid_record(format!(
                "unknown record version {}",
                file.version
            )));
        }
        let entries = file
            .hidden
            .into_iter()
            .map(|e| {
                let who = WindowIdentity {
                    process: e.process,
                    class: e.class,
                };
                (e.window, (e.how, who))
            })
            .collect();
        Ok(Self { entries })
    }
}

/// A record that cannot be used, saying why.
fn invalid_record(message: String) -> Error {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}

/// The record as written to disk.
#[derive(Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    hidden: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    window: WindowId,
    how: HidingBehaviour,
    process: u32,
    class: String,
}

#[cfg(test)]
#[path = "tests/hidden_tests.rs"]
mod tests;
