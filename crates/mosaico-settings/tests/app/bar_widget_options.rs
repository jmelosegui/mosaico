//! The options of an open widget on the Widgets tab of the Bar page.

use crate::bar::{FILE, bar_file};
use crate::bar_widgets::widgets;
use crate::support::*;
use eframe::egui::Key;

#[test]
fn the_format_of_an_open_widget_is_saved() {
    // Arrange
    let (store, _) = bar_file("bar-widget-format", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    widgets(&mut h);

    // Act
    click(&mut h, Role::Button, "Clock options");
    let format = h
        .get_all_by_role_and_label(Role::TextInput, "Format")
        .next()
        .unwrap();
    format.click();
    h.run();
    h.key_press(Key::End);
    h.get_by_role_and_label(Role::TextInput, "Format")
        .type_text("%H");
    h.run();
    click(&mut h, Role::Button, "Apply");

    // Assert
    let bar = store.load_bar();
    assert!(
        bar.left
            .iter()
            .any(|w| w.format().is_some_and(|f| f.ends_with("%H")))
    );
}

#[test]
fn the_format_field_shows_the_time_its_format_writes() {
    // Arrange
    let (store, _) = bar_file("bar-widget-example", FILE);
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    widgets(&mut h);
    click(&mut h, Role::Button, "Clock options");
    h.get_by_label("12:57:03");

    // Act
    h.get_by_role_and_label(Role::TextInput, "Format").click();
    h.run();
    h.key_press(Key::End);
    h.get_by_role_and_label(Role::TextInput, "Format")
        .type_text(" %Y");
    h.run();
    h.run();

    // Assert
    h.get_by_label("12:57:03 2026");
}

#[test]
fn an_open_widget_can_be_removed_from_the_bar() {
    // Arrange
    let (store, path) = bar_file("bar-widget-remove", FILE);
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Bar");
    widgets(&mut h);
    click(&mut h, Role::Button, "Clock options");

    // Act
    click(&mut h, Role::Button, "Remove Clock");
    h.get_by_label("1 unsaved change");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let kinds: Vec<_> = store
        .load_bar()
        .left
        .iter()
        .map(|w| w.kind().name())
        .collect();
    assert_eq!(kinds, ["workspaces"]);
    let written = std::fs::read_to_string(path).unwrap();
    assert!(written.starts_with("# my bar\n"), "{written}");
}
