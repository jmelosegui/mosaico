//! The Bar page, which edits `bar.toml`.

use crate::support::*;

const FILE: &str = "\
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

fn bar_file(test: &str, text: &str) -> (ConfigStore, PathBuf) {
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
fn a_widget_moved_up_is_saved_in_its_new_place() {
    // Arrange
    let (store, path) = bar_file("bar-widget-order", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");

    // Act
    click(&mut h, Role::Button, "Move Clock up");
    h.get_by_label("1 unsaved change");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(path).unwrap();
    let clock = written
        .find("# then the time\n[[left]]\ntype = \"clock\"")
        .unwrap();
    assert!(clock < written.find("type = \"workspaces\"").unwrap());
    let left: Vec<_> = store
        .load_bar()
        .left
        .iter()
        .map(|w| w.kind().name())
        .collect();
    assert_eq!(left, ["clock", "workspaces"]);
}

#[test]
fn a_widget_moved_to_the_other_side_and_switched_off_is_saved() {
    // Arrange
    let (store, _) = bar_file("bar-widget-across", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");

    // Act
    click(&mut h, Role::Button, "Move Clock to the right side");
    h.run();
    h.get_by_label("1 unsaved change");
    h.get_by_role_and_label(Role::Switch, "Clock")
        .scroll_to_me();
    h.run();
    click(&mut h, Role::Switch, "Clock");
    h.get_by_label("1 unsaved change");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let bar = store.load_bar();
    assert_eq!(bar.left.len(), 1);
    let clock = bar.right.last().unwrap();
    assert_eq!(clock.kind().name(), "clock");
    assert!(!clock.enabled());
}

#[test]
fn revert_drops_the_edits_of_the_bar_and_leaves_its_file_alone() {
    // Arrange
    let (store, path) = bar_file("bar-revert", FILE);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    click(&mut h, Role::Switch, "Show the bar");
    click(&mut h, Role::Button, "Move Clock up");
    h.get_by_label("2 unsaved changes");

    // Act
    click(&mut h, Role::Button, "Revert");

    // Assert
    h.get_by_label("No unsaved changes");
    assert!(
        h.query_by_role_and_label(Role::Button, "Move Clock up")
            .is_some()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
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

#[test]
fn search_finds_bar_settings_and_the_widget_order() {
    fn empty<'a>(h: &'a Harness<'static, SettingsApp>) -> egui_kittest::Node<'a> {
        h.get_all_by_role(Role::TextInput)
            .find(|n| n.accesskit_node().value().is_none_or(|v| v.is_empty()))
            .unwrap()
    }
    // The page has text fields of its own, so the search is the empty one.
    let mut h = stopped();
    open(&mut h, "Bar");
    empty(&h).click();
    h.run();
    empty(&h).type_text("opacity");
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Background opacity");
    assert!(h.query_by_label("Left widgets").is_none());
}
