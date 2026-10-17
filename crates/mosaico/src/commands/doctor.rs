use std::path::Path;

use mosaico_core::config::{ConfigStore, ThemeCheck, ThemeConfig};

use super::context::{Context, Exit, say};
use super::{doctor_animations, doctor_bar, doctor_runtime};

/// ANSI escape helpers for doctor output.
const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";
const FIXED: &str = "\x1b[36m[fixed]\x1b[0m";

pub fn execute(cx: &mut Context) -> Exit {
    (cx.logo)(&mut *cx.out);
    say!(cx, "");
    check_config_dir(&cx.config);
    check_config_file(&cx.config);
    check_theme(&cx.config);
    check_keybindings_file(&cx.config);
    check_keybinding_keys(&cx.config);
    check_rules_file(&cx.config);
    doctor_runtime::check_rules_cache_age(&cx.config);
    check_user_rules_file(&cx.config);
    doctor_bar::check_bar(cx);
    doctor_runtime::check_autostart(cx);
    doctor_runtime::check_daemon(cx);
    doctor_runtime::check_monitors(cx);
    doctor_animations::check_animations(cx);
    println!();
    Exit::Success
}

fn check_config_dir(store: &ConfigStore) {
    let dir = store.dir();
    if dir.is_dir() {
        println!("  {OK} Config directory exists ({})", dir.display());
        return;
    }
    match std::fs::create_dir_all(dir) {
        Ok(()) => {
            println!("  {FIXED} Created config directory ({})", dir.display());
        }
        Err(e) => {
            println!("  {FAIL} Config directory missing and could not create it: {e}");
        }
    }
}

/// Checks that a TOML config file exists and parses without errors.
fn check_toml_file(name: &str, path: &Path, try_load: impl FnOnce() -> mosaico_core::Result<()>) {
    println!("{}", toml_file_line(name, path, try_load));
}

/// The line [`check_toml_file`] prints for the file `name` at `path`.
pub(super) fn toml_file_line(
    name: &str,
    path: &Path,
    try_load: impl FnOnce() -> mosaico_core::Result<()>,
) -> String {
    if !path.exists() {
        return format!("  {WARN} {name} not found (using defaults)");
    }
    match try_load() {
        Ok(()) => format!("  {OK} {name} is valid"),
        Err(e) => format!("  {FAIL} {name}: {e}"),
    }
}

fn check_config_file(store: &ConfigStore) {
    check_toml_file("config.toml", &store.config_path(), || {
        store.try_load_config().map(|_| ())
    });
}

fn check_theme(store: &ConfigStore) {
    println!("{}", theme_line(&store.load_config().theme));
}

/// What doctor says about `theme`: the theme in use, or what an unknown
/// name or flavor falls back to.
fn theme_line(theme: &ThemeConfig) -> String {
    match theme.check() {
        ThemeCheck::Known(known) => {
            // A flavor left out reads as empty: name the one it stands for.
            let flavor = match theme.flavor.as_str() {
                "" => known.flavor_name().to_string(),
                written => written.to_ascii_lowercase(),
            };
            format!("  {OK} Theme: {} {flavor}", known.name())
        }
        ThemeCheck::UnknownFlavor(fallback) => format!(
            "  {WARN} Unknown flavor \"{}\", falling back to {}",
            theme.flavor,
            fallback.flavor_name()
        ),
        ThemeCheck::UnknownName(fallback) => format!(
            "  {WARN} Unknown theme \"{}\", falling back to {} {}",
            theme.name,
            fallback.name(),
            fallback.flavor_name()
        ),
    }
}

fn check_keybindings_file(store: &ConfigStore) {
    check_toml_file("keybindings.toml", &store.keybindings_path(), || {
        store.try_load_keybindings().map(|_| ())
    });
}

/// Flags the bindings whose key name the daemon would skip. The names
/// are checked in core, so this agrees with the daemon on every platform.
fn check_keybinding_keys(store: &ConfigStore) {
    let bindings = store.load_keybindings();
    let mut bad: Vec<String> = Vec::new();
    for kb in &bindings {
        if kb.combo().is_err() {
            bad.push(kb.key.clone());
        }
    }
    if bad.is_empty() {
        println!(
            "  {OK} All {} keybinding(s) resolve to valid key codes",
            bindings.len()
        );
    } else {
        println!(
            "  {FAIL} {} keybinding(s) have unknown keys: {}",
            bad.len(),
            bad.join(", ")
        );
    }
}

fn check_rules_file(store: &ConfigStore) {
    check_toml_file("rules.toml", &store.rules_path(), || {
        store.try_load_rules().map(|_| ())
    });
}

fn check_user_rules_file(store: &ConfigStore) {
    check_toml_file("user-rules.toml", &store.user_rules_path(), || {
        store.try_load_user_rules().map(|_| ())
    });
}

#[cfg(test)]
#[path = "tests/doctor_tests.rs"]
mod tests;
