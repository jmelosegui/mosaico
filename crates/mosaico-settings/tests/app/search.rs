//! The search looks on every page, not only the one shown.

use eframe::egui::Key;

use crate::support::*;

fn searched(text: &str) -> Harness<'static, SettingsApp> {
    let mut h = stopped();
    search(&mut h, text);
    h.run();
    h
}

#[test]
fn search_finds_settings_on_every_page() {
    // The window opens on General, which has no gap.
    let h = searched("gap");

    h.get_by_role_and_label(Role::SpinButton, "Gap");
    h.get_by_role_and_label(Role::SpinButton, "Item gap");
    assert_eq!(h.query_all_by_label("Theme").count(), 0);
}

#[test]
fn search_finds_keybindings_by_their_action() {
    let h = searched("focus down");

    h.get_by_role_and_label(Role::Button, "Change Focus down");
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus up")
            .is_none()
    );
}

#[test]
fn search_finds_a_setting_by_its_key_in_the_file() {
    let h = searched("corner_style");

    h.get_by_label("Corner style");
}

#[test]
fn search_finds_a_bar_widget_by_its_name() {
    let h = searched("clock");

    h.get_by_role_and_label(Role::Button, "Move Clock");
}

#[test]
fn clearing_the_search_shows_the_page_again() {
    let mut h = stopped();
    open(&mut h, "Keybindings");
    search(&mut h, "gap");
    h.run();
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus down")
            .is_none()
    );

    click(&mut h, Role::Button, "Clear search");

    h.get_by_role_and_label(Role::Button, "Change Focus down");
    assert!(h.query_by_role_and_label(Role::SpinButton, "Gap").is_none());
}

#[test]
fn escape_clears_the_search() {
    let mut h = searched("gap");

    h.key_press(Key::Escape);
    h.run();

    assert!(h.query_by_role_and_label(Role::SpinButton, "Gap").is_none());
    assert!(h.query_all_by_label("Theme").count() > 0);
}

#[test]
fn a_page_picked_while_searching_is_shown_when_the_search_is_cleared() {
    let mut h = searched("gap");

    open(&mut h, "Bar");
    h.get_by_role_and_label(Role::SpinButton, "Gap");
    click(&mut h, Role::Button, "Clear search");

    h.get_by_role_and_label(Role::Tab, "Appearance");
    assert!(h.query_by_role_and_label(Role::SpinButton, "Gap").is_none());
}

#[test]
fn a_new_search_starts_at_the_top() {
    let mut h = searched("focus");
    open(&mut h, "Keybindings");
    click(&mut h, Role::Button, "Clear search");

    search(&mut h, "e");
    h.run();

    let theme = h.get_by_role_and_label(Role::ComboBox, "Theme").rect();
    assert!(
        theme.top() > 0.0,
        "Theme is scrolled out of view at {theme:?}"
    );
}
