//! The Widgets tab of the Bar page: the order, options and removal of
//! the widgets on each side.

use crate::bar::{FILE, bar_file};
use crate::support::*;
use eframe::egui::Key;

pub(crate) fn widgets(h: &mut Harness<'static, SettingsApp>) {
    click(h, Role::Tab, "Widgets");
}

fn press(h: &mut Harness<'static, SettingsApp>, widget: &str, key: Key) {
    h.get_by_role_and_label(Role::Button, &format!("Move {widget}"))
        .focus();
    h.run();
    h.key_press(key);
    h.run();
}

#[test]
fn a_widget_moved_up_with_its_grip_is_saved_in_its_new_place() {
    // Arrange
    let (store, path) = bar_file("bar-widget-order", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    widgets(&mut h);

    // Act
    press(&mut h, "Clock", Key::ArrowUp);
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
    widgets(&mut h);

    // Act
    press(&mut h, "Clock", Key::ArrowRight);
    h.get_by_label("1 unsaved change");
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
fn a_widget_can_be_added_to_a_side() {
    // Arrange
    let (store, _) = bar_file("bar-widget-add", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    widgets(&mut h);

    // Act
    h.get_all_by_role_and_label(Role::Button, "Add widget")
        .next()
        .unwrap()
        .click();
    h.run();
    click(&mut h, Role::Button, "CPU");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let kinds: Vec<_> = store
        .load_bar()
        .left
        .iter()
        .map(|w| w.kind().name())
        .collect();
    assert_eq!(kinds.last(), Some(&"cpu"));
}

#[test]
fn revert_drops_the_edits_of_the_bar_and_leaves_its_file_alone() {
    // Arrange
    let (store, path) = bar_file("bar-revert", FILE);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    click(&mut h, Role::Switch, "Show the bar");
    widgets(&mut h);
    press(&mut h, "Clock", Key::ArrowUp);
    h.get_by_label("2 unsaved changes");

    // Act
    click(&mut h, Role::Button, "Revert");

    // Assert
    h.get_by_label("No unsaved changes");
    widgets(&mut h);
    assert!(
        h.query_by_role_and_label(Role::Button, "Move Clock")
            .is_some()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
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
}
