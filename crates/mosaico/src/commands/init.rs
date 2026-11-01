use mosaico_core::config::{self, ConfigStore, DefaultFile};
use mosaico_core::system::AutostartStatus;

use super::autostart;
use super::context::{Context, Exit, complain, say};

/// Creates the default configuration files in the instance's config
/// directory, `~/.config/mosaico/` for the default instance.
///
/// Generates `config.toml`, `keybindings.toml`, `rules.toml`,
/// `user-rules.toml`, and `bar.toml` with comments explaining every
/// option. Existing files are not overwritten. On first run, prompts
/// the user to enable autostart, see [`offers_autostart`].
pub fn execute(cx: &mut Context) -> Exit {
    let store = cx.config.clone();
    check_rules_migration(cx, &store);

    let Some(created_config) = write_files(cx, &store) else {
        return Exit::Failure;
    };
    if offers_autostart(cx, created_config) {
        prompt_autostart(cx);
    }

    say!(
        cx,
        "\nEdit these files to customize layout, keybindings, window rules, and the status bar."
    );
    say!(
        cx,
        "Add personal window rules in user-rules.toml (community rules in rules.toml are"
    );
    say!(
        cx,
        "downloaded automatically and will be overwritten on daemon startup)."
    );
    Exit::Success
}

/// Writes the config files `store` does not have yet, saying what became
/// of each. Returns whether `config.toml` was written, or `None` when
/// the config directory cannot be created.
fn write_files(cx: &mut Context, store: &ConfigStore) -> Option<bool> {
    let files = match store.write_missing_defaults() {
        Ok(files) => files,
        Err(e) => {
            complain!(cx, "Error: could not create {}: {e}", store.dir().display());
            return None;
        }
    };
    let config_path = store.config_path();
    let mut created_config = false;
    for file in files {
        match file {
            DefaultFile::Created(path) => {
                created_config |= path == config_path;
                say!(cx, "Created {}", path.display());
            }
            DefaultFile::Existed(path) => say!(cx, "Already exists: {}", path.display()),
            DefaultFile::Failed(path, e) => {
                complain!(cx, "Error: could not write {}: {e}", path.display());
            }
        }
    }
    Some(created_config)
}

/// Whether this is a first run, where `init` offers to start mosaico at
/// sign-in: `init` just wrote `config.toml`, or the daemon wrote it on a
/// fresh install that started the daemon first, and nobody has edited it
/// or registered autostart since.
fn offers_autostart(cx: &Context, created_config: bool) -> bool {
    if created_config {
        return true;
    }
    let untouched = std::fs::read_to_string(cx.config.config_path())
        .is_ok_and(|text| text == config::template::generate_config());
    untouched && matches!(autostart::check(cx), Ok(AutostartStatus::Disabled))
}

/// Asks the user whether to register Mosaico for automatic startup.
///
/// Default is No (`[y/N]`) so pressing Enter skips autostart.
fn prompt_autostart(cx: &mut Context) {
    let _ = write!(
        cx.out,
        "\nWould you like Mosaico to start automatically when Windows starts? [y/N]: "
    );
    let _ = cx.out.flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y") {
        match autostart::register(cx) {
            Ok(()) => say!(cx, "Autostart enabled."),
            Err(e) => complain!(cx, "Warning: could not enable autostart: {e}"),
        }
    }
}

/// Warns if `rules.toml` has custom rules that should move to `user-rules.toml`.
///
/// Community rules now overwrite `rules.toml` on every daemon startup.
/// If the user previously added custom rules there, they need to migrate
/// them to `user-rules.toml` to avoid losing them.
fn check_rules_migration(cx: &mut Context, store: &ConfigStore) {
    if !store.rules_path().exists() || store.user_rules_path().exists() {
        return;
    }

    let default_count = config::default_rules().len();
    let Ok(current) = store.try_load_rules() else {
        return;
    };

    if current.len() > default_count {
        say!(
            cx,
            "\x1b[33m[notice]\x1b[0m rules.toml contains custom rules."
        );
        say!(
            cx,
            "         Community rules now overwrite this file on daemon startup."
        );
        say!(
            cx,
            "         Move your custom rules to user-rules.toml to keep them."
        );
    }
}

#[cfg(test)]
#[path = "tests/init_tests.rs"]
mod tests;
