//! What `mosaico doctor` says about launch bindings, and about key
//! combinations bound more than once.

use mosaico_core::Result;
use mosaico_core::config::{ConfigStore, Keybinding, LaunchBinding, conflicts_with_launches};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";

/// Reports the `[[launch]]` entries of `keybindings.toml` and the
/// combinations two bindings share. Says nothing without the file.
pub fn check_launch_bindings(store: &ConfigStore) {
    if !store.keybindings_path().exists() {
        return;
    }
    let keybindings = store.load_keybindings();
    for line in launch_lines(&keybindings, store.try_load_launches()) {
        println!("{line}");
    }
}

/// What doctor says about the launch bindings `entries` read next to
/// `keybindings`: each one the daemon would skip, a count of the good
/// ones, and each combination bound more than once.
fn launch_lines(
    keybindings: &[Keybinding],
    entries: Result<Vec<Result<LaunchBinding>>>,
) -> Vec<String> {
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => return vec![format!("  {FAIL} Launch bindings: {e}")],
    };
    let mut lines = Vec::new();
    let mut launches = Vec::new();
    for entry in entries {
        match entry {
            Ok(launch) if launch.combo().is_err() => lines.push(format!(
                "  {FAIL} {} has an unknown key: {}",
                launch.name(),
                launch.key
            )),
            Ok(launch) => launches.push(launch),
            Err(e) => lines.push(format!("  {FAIL} {e}")),
        }
    }
    if !launches.is_empty() && lines.is_empty() {
        lines.push(format!(
            "  {OK} All {} launch binding(s) are valid",
            launches.len()
        ));
    }
    for conflict in conflicts_with_launches(keybindings, &launches) {
        let names: Vec<String> = conflict
            .bindings
            .iter()
            .map(|&i| keybindings[i].action.to_string())
            .chain(conflict.launches.iter().map(|&i| launches[i].name()))
            .collect();
        lines.push(format!(
            "  {WARN} {} is bound more than once, and only the first works: {}",
            conflict.combo,
            names.join(", ")
        ));
    }
    lines
}

#[cfg(test)]
#[path = "tests/doctor_launch_tests.rs"]
mod tests;
