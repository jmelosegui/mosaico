use crate::config::bar::BarConfig;

use super::ConfigStore;
use super::merge_bar_mouse::append_missing_bar_mouse_hint;

impl ConfigStore {
    /// Loads bar config and appends any missing default widgets to the user's file.
    ///
    /// Compares the user's `[[left]]` and `[[right]]` widget lists against the
    /// built-in defaults by widget type. Any default widget type not already
    /// present in the user's file is appended so new widgets from future versions
    /// are picked up automatically, without overwriting anything the user has set.
    ///
    /// A file without a `[mouse]` section also gets a comment block about
    /// it, which changes nothing it says.
    ///
    /// Without a file, returns the defaults and creates nothing. A file
    /// that is not TOML is left alone, and the defaults are returned. One
    /// that is TOML but holds a value the bar cannot take, such as a
    /// height that is not a number, still gets the `[mouse]` comment block
    /// when it has no such section, and the defaults are returned. When
    /// appending widgets fails, returns what the file has.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_bar_widgets(&self) -> BarConfig {
        use crate::config::bar::WidgetConfig;
        use serde::Serialize;

        let path = self.bar_path();
        if !path.exists() {
            return self.load_bar();
        }

        match append_missing_bar_mouse_hint(&path) {
            Ok(true) => crate::log_info!(
                "Appended [mouse] hint to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the [mouse] hint to bar.toml: {e}"),
        }

        let mut user = match self.try_load_bar() {
            Ok(cfg) => cfg,
            Err(e) => {
                crate::log_warn!("Using the default bar: {e}");
                return BarConfig::default();
            }
        };

        let defaults = BarConfig::default();

        // Find widget types (by enum discriminant) present in defaults but missing
        // from the user's left/right lists. Users may intentionally remove widgets,
        // but a type that never existed before must be a new default.
        let missing_left: Vec<&WidgetConfig> = defaults
            .left
            .iter()
            .filter(|d| {
                !user
                    .left
                    .iter()
                    .any(|u| std::mem::discriminant(u) == std::mem::discriminant(*d))
            })
            .collect();

        let missing_right: Vec<&WidgetConfig> = defaults
            .right
            .iter()
            .filter(|d| {
                !user
                    .right
                    .iter()
                    .any(|u| std::mem::discriminant(u) == std::mem::discriminant(*d))
            })
            .collect();

        if missing_left.is_empty() && missing_right.is_empty() {
            return user;
        }

        // Serialize missing entries as TOML using a wrapper struct.
        #[derive(Serialize)]
        struct BarSideDiff<'a> {
            #[serde(skip_serializing_if = "Vec::is_empty")]
            left: Vec<&'a WidgetConfig>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            right: Vec<&'a WidgetConfig>,
        }

        let diff = BarSideDiff {
            left: missing_left.clone(),
            right: missing_right.clone(),
        };

        let toml_addition = match toml::to_string(&diff) {
            Ok(s) => {
                format!("\n# Added automatically — new defaults from this version of mosaico\n{s}")
            }
            Err(e) => {
                crate::log_warn!("Could not serialize missing bar widgets: {e}");
                return user;
            }
        };

        match std::fs::OpenOptions::new().append(true).open(&path) {
            Ok(mut f) => {
                use std::io::Write;
                if let Err(e) = f.write_all(toml_addition.as_bytes()) {
                    crate::log_warn!("Could not append missing bar widgets: {e}");
                    return user;
                }
            }
            Err(e) => {
                crate::log_warn!("Could not open bar.toml for appending: {e}");
                return user;
            }
        }

        let total = missing_left.len() + missing_right.len();
        crate::log_info!("Appended {total} missing default bar widget(s) to bar.toml");

        // Return merged config.
        user.left.extend(missing_left.into_iter().cloned());
        user.right.extend(missing_right.into_iter().cloned());
        user
    }
}

#[cfg(test)]
#[path = "tests/merge_bar_tests.rs"]
mod tests;
