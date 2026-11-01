//! Apply, and following the file when nothing is unsaved.

use crate::support::*;

#[test]
fn apply_writes_the_edits_to_config_toml_and_keeps_its_comments() {
    // Arrange
    let store = folder("apply-writes");
    let file = "# my setup\n[logging]\nlevel = \"warn\" # quieter\n";
    std::fs::write(store.config_path(), file).unwrap();
    let path = store.config_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    click(&mut h, Role::Switch, "Write a log file");

    // Act
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, format!("{file}enabled = true\n"));
    assert_eq!(h.state().applied(), Some(&Applied::Saved));
    assert_eq!(h.state().saved(), h.state().edit());
    h.get_by_label("Changes saved");
}

#[test]
fn apply_turns_start_at_sign_in_on_through_the_system() {
    let sign_in = SignIn::default();
    let mut h = window(folder("apply-sign-in"), DaemonState::Stopped, &sign_in);
    click(&mut h, Role::Switch, "Start at sign-in");

    click(&mut h, Role::Button, "Apply");

    assert_eq!(*sign_in.asked.borrow(), [true]);
    assert!(h.state().saved().start_at_sign_in);
    h.get_by_label("Changes saved");
}

#[test]
fn a_failed_apply_says_why_and_keeps_the_edit_that_failed() {
    // Arrange
    let store = folder("apply-fails");
    let path = store.config_path();
    let sign_in = SignIn {
        fails: true,
        ..SignIn::default()
    };
    let mut h = window(store, DaemonState::Stopped, &sign_in);
    click(&mut h, Role::Switch, "Write a log file");
    click(&mut h, Role::Switch, "Start at sign-in");

    // Act
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_label("Could not save: the system said no");
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, "[logging]\nenabled = true\n");
    assert!(!h.state().saved().start_at_sign_in);
    assert!(h.state().edit().start_at_sign_in);
}

#[test]
fn a_file_the_daemon_would_not_load_is_shown_and_not_saved_over() {
    // Arrange
    let store = folder("apply-broken");
    let broken = "[animations]\neasing = \"wobble\"\n";
    std::fs::write(store.config_path(), broken).unwrap();
    let path = store.config_path();

    // Act
    let h = window(store, DaemonState::Stopped, &SignIn::default());

    // Assert
    h.get_by_label("config.toml has a problem, so it can't be edited here");
    assert!(
        h.get_by_role_and_label(Role::Button, "Apply")
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(h.state().edit(), &saved());
    assert_eq!(std::fs::read_to_string(path).unwrap(), broken);
}

#[test]
fn an_edit_made_in_an_editor_shows_up_while_nothing_is_unsaved() {
    // Arrange
    let store = folder("apply-outside-edit");
    let path = store.config_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());

    // Act
    std::fs::write(path, "[logging]\nenabled = true\n").unwrap();
    for _ in 0..20 {
        h.step();
    }

    // Assert
    assert_eq!(logging(&h), &SettingValue::Bool(true));
    assert_eq!(h.state().saved(), h.state().edit());
}
