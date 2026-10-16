use mosaico_core::config::{self, ConfigStore};

use super::autostart;
use super::context::{Context, Exit};

/// Creates the default configuration files in the instance's config
/// directory, `~/.config/mosaico/` for the default instance.
///
/// Generates `config.toml`, `keybindings.toml`, `rules.toml`,
/// `user-rules.toml`, and `bar.toml` with comments explaining every
/// option. Existing files are not overwritten. On first run, prompts
/// the user to enable autostart.
pub fn execute(cx: &mut Context) -> Exit {
    let store = cx.config.clone();
    let dir = store.dir();

    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("Error: could not create {}: {e}", dir.display());
        return Exit::Failure;
    }

    let is_first_run = !store.config_path().exists();

    check_rules_migration(&store);

    write_if_missing(&store.config_path(), &config::template::generate_config());
    write_if_missing(
        &store.keybindings_path(),
        &config::template::generate_keybindings(),
    );
    write_if_missing(&store.rules_path(), &config::template::generate_rules());
    write_if_missing(
        &store.user_rules_path(),
        &config::template::generate_user_rules(),
    );
    write_if_missing(&store.bar_path(), &config::template::generate_bar());

    if is_first_run {
        prompt_autostart(cx);
    }

    println!(
        "\nEdit these files to customize layout, keybindings, window rules, and the status bar."
    );
    println!("Add personal window rules in user-rules.toml (community rules in rules.toml are");
    println!("downloaded automatically and will be overwritten on daemon startup).");
    Exit::Success
}

/// Asks the user whether to register Mosaico for automatic startup.
///
/// Default is No (`[y/N]`) so pressing Enter skips autostart.
fn prompt_autostart(cx: &mut Context) {
    use std::io::Write;
    print!("\nWould you like Mosaico to start automatically when Windows starts? [y/N]: ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y") {
        match autostart::register(cx) {
            Ok(()) => println!("Autostart enabled."),
            Err(e) => eprintln!("Warning: could not enable autostart: {e}"),
        }
    }
}

/// Warns if `rules.toml` has custom rules that should move to `user-rules.toml`.
///
/// Community rules now overwrite `rules.toml` on every daemon startup.
/// If the user previously added custom rules there, they need to migrate
/// them to `user-rules.toml` to avoid losing them.
fn check_rules_migration(store: &ConfigStore) {
    if !store.rules_path().exists() || store.user_rules_path().exists() {
        return;
    }

    let default_count = config::default_rules().len();
    let Ok(current) = store.try_load_rules() else {
        return;
    };

    if current.len() > default_count {
        println!("\x1b[33m[notice]\x1b[0m rules.toml contains custom rules.");
        println!("         Community rules now overwrite this file on daemon startup.");
        println!("         Move your custom rules to user-rules.toml to keep them.");
    }
}

/// Writes content to a file only if it doesn't already exist.
fn write_if_missing(path: &std::path::Path, content: &str) {
    if path.exists() {
        println!("Already exists: {}", path.display());
        return;
    }

    match std::fs::write(path, content) {
        Ok(()) => println!("Created {}", path.display()),
        Err(e) => eprintln!("Error: could not write {}: {e}", path.display()),
    }
}
