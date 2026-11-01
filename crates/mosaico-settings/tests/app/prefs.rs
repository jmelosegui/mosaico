//! What the window remembers between runs, in `settings-app.toml`.

use eframe::egui::{Rect, pos2, vec2};

use crate::support::*;

/// Says the window is `width` by `height` points, and whether it fills
/// the screen, as the system would.
fn resize(h: &mut Harness<'static, SettingsApp>, width: f32, height: f32, maximized: bool) {
    let root = h.input_mut().viewports.entry(ViewportId::ROOT).or_default();
    root.inner_rect = Some(Rect::from_min_size(pos2(40.0, 30.0), vec2(width, height)));
    root.maximized = Some(maximized);
    h.run();
}

#[test]
fn opens_on_the_page_it_showed_last() {
    let store = folder("prefs-page");
    std::fs::write(store.settings_app_path(), "page = \"Bar\"\n").unwrap();

    let h = window(store, DaemonState::Stopped, &SignIn::default());

    h.get_by_role_and_label(Role::Switch, "Show the bar");
}

#[test]
fn remembers_the_page_and_the_size_it_closes_with() {
    // Arrange
    let store = folder("prefs-close");
    let mut h = window(store.clone(), DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Mouse");
    resize(&mut h, 1100.0, 760.0, false);

    // Act
    h.state().remember();

    // Assert
    let saved = store.load_settings_app();
    assert_eq!(saved.page, "Mouse");
    assert_eq!((saved.width, saved.height), (1100.0, 760.0));
    assert!(!saved.maximized);
}

#[test]
fn a_window_filling_the_screen_keeps_the_size_to_go_back_to() {
    // Arrange
    let mut h = stopped();
    resize(&mut h, 1100.0, 760.0, false);

    // Act
    resize(&mut h, 2560.0, 1400.0, true);

    // Assert
    let remembered = h.state().remembered();
    assert!(remembered.maximized);
    assert_eq!((remembered.width, remembered.height), (1100.0, 760.0));
}
