use crate::config::reload::test_support::{temp_store, write_at};

use super::*;

/// A keybindings file with one action and one application on Alt+Shift+T.
const WITH_TERMINAL: &str = "[[keybinding]]\naction = \"focus-down\"\nkey = \"J\"\n\
    modifiers = [\"alt\"]\n\n[[launch]]\ncommand = \"wt\"\nkey = \"T\"\n\
    modifiers = [\"alt\", \"shift\"]\n";

#[test]
fn an_application_added_to_keybindings_toml_while_the_daemon_runs_is_reloaded() {
    // Arrange -- the settings app saves a new application to the file.
    let store = temp_store("reload-keybindings");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.keybindings_path(), WITH_TERMINAL, 1);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- the new keys reach the daemon, so a press of them works.
    let shown = format!("{reloads:?}");
    assert_eq!(reloads.len(), 1, "{shown}");
    assert!(shown.starts_with("[Keybindings"), "{shown}");
    assert!(shown.contains("Focus(Down)"), "{shown}");
    assert!(shown.contains("command: \"wt\""), "{shown}");
}
