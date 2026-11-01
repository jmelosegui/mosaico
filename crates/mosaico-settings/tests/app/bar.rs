//! The Bar page, which edits `bar.toml`.

use crate::support::*;

pub(crate) const FILE: &str = "\
# my bar\n\
height = 40 # tall\nright = []\n\
\n\
# first\n\
[[left]]\n\
type = \"workspaces\"\n\
\n\
# then the time\n\
[[left]]\n\
type = \"clock\"\n";

pub(crate) fn bar_file(test: &str, text: &str) -> (ConfigStore, PathBuf) {
    let store = folder(test);
    let path = store.bar_path();
    std::fs::write(&path, text).unwrap();
    (store, path)
}

#[test]
fn a_bar_setting_is_saved_to_bar_toml_keeping_its_comments() {
    // Arrange
    let (store, path) = bar_file("bar-setting", FILE);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");

    // Act
    click(&mut h, Role::Switch, "Show the bar");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        written,
        FILE.replace("right = []\n", "right = []\nenabled = false\n")
    );
    h.get_by_label("Changes saved");
}

#[test]
fn a_monitor_unticked_is_saved_as_the_list_of_the_others() {
    // Arrange
    let (store, path) = bar_file("bar-monitors", FILE);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");

    // Act
    click(&mut h, Role::CheckBox, "Monitor 2");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    assert!(written.contains("monitors = [0]"), "{written}");
}

#[test]
fn a_click_action_can_be_added_and_removed() {
    // Arrange
    let (store, _) = bar_file("bar-clicks", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    h.get_all_by_role_and_label(Role::Tab, "Mouse")
        .last()
        .unwrap()
        .click();
    h.run();

    // Act
    click(&mut h, Role::Button, "Add click action");
    h.get_by_label("1 unsaved change");
    click(&mut h, Role::Button, "Remove click action 1");
    click(&mut h, Role::Button, "Add click action");
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_label("Changes saved");
}

#[test]
fn the_footer_counts_the_edits_of_both_files_and_apply_saves_both() {
    // Arrange
    let (store, bar) = bar_file("bar-both-files", FILE);
    let config = store.config_path();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    click(&mut h, Role::Switch, "Write a log file");
    open(&mut h, "Bar");

    // Act
    click(&mut h, Role::Switch, "Show the bar");
    h.get_by_label("2 unsaved changes");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(config).unwrap();
    assert_eq!(written, "[logging]\nenabled = true\n");
    assert!(
        std::fs::read_to_string(bar)
            .unwrap()
            .contains("enabled = false")
    );
    h.get_by_label("Changes saved");
}

#[test]
fn a_bar_file_the_daemon_would_not_load_shows_its_problem_on_the_bar_page_only() {
    // Arrange
    let broken = "height = \"tall\"\n";
    let (store, path) = bar_file("bar-broken", broken);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());

    // Act
    click(&mut h, Role::Switch, "Write a log file");
    open(&mut h, "Bar");

    // Assert
    h.get_by_label("bar.toml has a problem, so it can't be edited here");
    assert!(
        h.get_by_role_and_label(Role::Switch, "Show the bar")
            .accesskit_node()
            .is_disabled()
    );
    open(&mut h, "General");
    assert!(
        h.query_by_label("bar.toml has a problem, so it can't be edited here")
            .is_none()
    );
    h.get_by_label("1 unsaved change");
    click(&mut h, Role::Button, "Apply");
    assert_eq!(std::fs::read_to_string(path).unwrap(), broken);
    h.get_by_label("Changes saved");
}
