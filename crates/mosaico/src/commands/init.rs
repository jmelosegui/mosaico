use mosaico_core::config;

/// Creates the default configuration files at `~/.config/mosaico/`.
///
/// Generates `config.toml`, `keybindings.toml`, `rules.toml`,
/// `user-rules.toml`, and `bar.toml` with comments explaining every
/// option. Existing files are not overwritten.
pub fn execute() {
    let Some(dir) = config::config_dir() else {
        eprintln!("Error: could not determine home directory.");
        std::process::exit(1);
    };

    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Error: could not create {}: {e}", dir.display());
        std::process::exit(1);
    }

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

    println!(
        "\nEdit these files to customize layout, keybindings, window rules, and the status bar."
    );
    println!("Add personal window rules in user-rules.toml (community rules in rules.toml are");
    println!("downloaded automatically and will be overwritten on daemon startup).");
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
