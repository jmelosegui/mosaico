use crate::config::{Keybinding, keybinding};

use super::ConfigStore;

impl ConfigStore {
    /// Loads keybindings and appends any missing defaults to the user's file.
    ///
    /// Compares the user's configured actions against the built-in defaults.
    /// Any default action not already bound by the user is appended to the
    /// keybindings file so new bindings from future versions are picked up
    /// automatically, without overwriting anything the user has configured.
    ///
    /// Without a file, returns the defaults and creates nothing. When the
    /// file cannot be parsed, returns the defaults and leaves it alone.
    /// When appending fails, returns what the file has.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_keybindings(&self) -> Vec<Keybinding> {
        let path = self.keybindings_path();
        if !path.exists() {
            return self.load_keybindings();
        }

        let user = match self.try_load_keybindings() {
            Ok(kb) => kb,
            Err(e) => {
                crate::log_warn!("Using the default keybindings: {e}");
                return keybinding::defaults();
            }
        };

        let defaults = keybinding::defaults();
        let missing: Vec<&Keybinding> = defaults
            .iter()
            .filter(|d| !user.iter().any(|u| u.action == d.action))
            .collect();

        if missing.is_empty() {
            return user;
        }

        // Append missing bindings to the file.
        let mut addition =
            String::from("\n# Added automatically — new defaults from this version of mosaico\n");
        for kb in &missing {
            addition.push_str(&keybinding_toml_entry(kb));
        }

        match std::fs::OpenOptions::new().append(true).open(&path) {
            Ok(mut f) => {
                use std::io::Write;
                if let Err(e) = f.write_all(addition.as_bytes()) {
                    crate::log_warn!("Could not append missing keybindings: {e}");
                    return user;
                }
            }
            Err(e) => {
                crate::log_warn!("Could not open keybindings file for appending: {e}");
                return user;
            }
        }

        crate::log_info!(
            "Appended {} missing default keybinding(s) to keybindings.toml",
            missing.len()
        );

        // Return the full merged set.
        let mut merged = user;
        merged.extend(missing.into_iter().cloned());
        merged
    }
}

/// Formats a single keybinding as a `[[keybinding]]` TOML entry.
pub(super) fn keybinding_toml_entry(kb: &Keybinding) -> String {
    let modifiers: Vec<String> = kb
        .modifiers
        .iter()
        .map(|m| {
            let s = match m {
                keybinding::Modifier::Alt => "alt",
                keybinding::Modifier::Shift => "shift",
                keybinding::Modifier::Ctrl => "ctrl",
                keybinding::Modifier::Win => "win",
            };
            format!("\"{s}\"")
        })
        .collect();
    format!(
        "\n[[keybinding]]\naction = \"{}\"\nkey = \"{}\"\nmodifiers = [{}]\n",
        kb.action,
        kb.key,
        modifiers.join(", ")
    )
}

#[cfg(test)]
#[path = "tests/merge_keybindings_tests.rs"]
mod tests;
