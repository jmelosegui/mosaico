use mosaico_core::config;

/// Creates the default configuration files at `~/.config/mosaico/`.
///
/// Generates `config.toml`, `keybindings.toml`, `rules.toml`,
/// `user-rules.toml`, and `bar.toml` with comments explaining every
/// option. Existing files are not overwritten. On first run, prompts
/// the user to enable autostart.
pub fn execute() {
    let Some(dir) = config::config_dir() else {
        eprintln!("Error: could not determine home directory.");
        std::process::exit(1);
    };

    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Error: could not create {}: {e}", dir.display());
        std::process::exit(1);
    }

    let is_first_run = !dir.join("config.toml").exists();

    check_rules_migration(&dir);

    write_if_missing(
        &dir.join("config.toml"),
        &config::template::generate_config(),
    );
    write_if_missing(
        &dir.join("keybindings.toml"),
        &config::template::generate_keybindings(),
    );
    write_if_missing(&dir.join("rules.toml"), &config::template::generate_rules());
    write_if_missing(
        &dir.join("user-rules.toml"),
        &config::template::generate_user_rules(),
    );
    write_if_missing(&dir.join("bar.toml"), &config::template::generate_bar());

    if is_first_run {
        prompt_autostart();
    }

    println!(
        "\nEdit these files to customize layout, keybindings, window rules, and the status bar."
    );
    println!("Add personal window rules in user-rules.toml (community rules in rules.toml are");
    println!("downloaded automatically and will be overwritten on daemon startup).");
}

/// Asks the user whether to register Mosaico for automatic startup.
///
/// Default is No (`[y/N]`) so pressing Enter skips autostart.
fn prompt_autostart() {
    use std::io::Write;
    print!("\nWould you like Mosaico to start automatically when Windows starts? [y/N]: ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y") {
        match mosaico_windows::autostart::enable() {
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
fn check_rules_migration(dir: &std::path::Path) {
    let rules_path = dir.join("rules.toml");
    let user_rules_path = dir.join("user-rules.toml");

    if !rules_path.exists() || user_rules_path.exists() {
        return;
    }

    let default_count = config::default_rules().len();
    let Ok(current) = config::try_load_rules() else {
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
