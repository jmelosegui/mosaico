//! Config files with chosen modification times, for the reload tests.

use std::fs::OpenOptions;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub(super) use crate::config::store::test_support::temp_store;

use super::ConfigReload;

/// A `config.toml` with a gap of 7.
pub(super) const GAP_7: &str = "[layout]\ngap = 7\n";

/// A file that is not valid TOML.
pub(super) const INVALID: &str = "not = [valid";

/// A rules file with one rule for the windows of class `class`.
pub(super) fn rule_for(class: &str) -> String {
    format!("[[rule]]\nmatch_class = \"{class}\"\nmanage = false\n")
}

/// Writes `content` to `path` and dates it `secs` seconds after the
/// epoch, so the test, not the clock's resolution, decides which writes
/// count as changes.
pub(super) fn write_at(path: &Path, content: &str, secs: u64) {
    std::fs::write(path, content).unwrap();
    touch_at(path, secs);
}

/// Dates the file at `path` `secs` seconds after the epoch, keeping what
/// it holds.
pub(super) fn touch_at(path: &Path, secs: u64) {
    let file = OpenOptions::new().write(true).open(path).unwrap();
    let at = SystemTime::UNIX_EPOCH + Duration::from_secs(secs);
    file.set_modified(at).unwrap();
}

/// What each reload is about, to compare reloads in one assertion.
pub(super) fn kinds(reloads: &[ConfigReload]) -> Vec<&'static str> {
    reloads
        .iter()
        .map(|reload| match reload {
            ConfigReload::Config(_) => "config",
            ConfigReload::Rules(_) => "rules",
            ConfigReload::Bar(_) => "bar",
            ConfigReload::Invalid { .. } => "invalid",
        })
        .collect()
}

/// The classes the rules of a rules reload match, in order.
pub(super) fn rule_classes(reload: &ConfigReload) -> Vec<String> {
    match reload {
        ConfigReload::Rules(rules) => rules
            .iter()
            .map(|rule| rule.match_class.clone().unwrap_or_default())
            .collect(),
        other => panic!("expected a rules reload, got {other:?}"),
    }
}
