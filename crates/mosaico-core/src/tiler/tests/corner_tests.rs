//! The corners the tiler gives the windows it manages, and gives back.

use crate::config::WindowRule;

use super::*;

#[test]
fn a_menu_that_closes_keeps_its_own_corners() {
    // #74
    // Arrange -- menus report a hide every time they close, and some
    // draw their own corners. The platform rules them out on its own.
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut menu = FakeWindow::app(0);
    menu.facts = None;
    menu.visible = false;
    h.desktop.windows.insert(wid(9), menu);

    // Act
    let effects = h.event(WindowEvent::Hidden { window: wid(9) }).to_vec();

    // Assert
    assert!(
        effects.is_empty(),
        "the corners of a window the tiler never managed were reset: {effects:?}"
    );
}

#[test]
fn a_window_the_rules_leave_alone_keeps_its_own_corners() {
    // #74
    // Arrange -- an application the rules do not tile.
    let rule = WindowRule {
        match_class: Some("Floating".into()),
        match_title: None,
        match_path: None,
        manage: false,
    };
    let settings = Settings {
        rules: vec![rule],
        ..Settings::default()
    };
    let mut h = Harness::with_settings(settings, 1, &[(1, 0)]);
    let mut floating = FakeWindow::app(0);
    if let Some(facts) = floating.facts.as_mut() {
        facts.class = "Floating".into();
    }
    h.desktop.windows.insert(wid(9), floating);
    h.event(WindowEvent::Created { window: wid(9) });

    // Act -- it hides to the tray, then closes.
    h.window(9).visible = false;
    let hidden = h.event(WindowEvent::Hidden { window: wid(9) }).to_vec();
    h.desktop.windows.remove(&wid(9));
    let closed = h.event(WindowEvent::Destroyed { window: wid(9) }).to_vec();

    // Assert
    assert!(
        hidden.is_empty(),
        "hiding a window the tiler never managed reset its corners: {hidden:?}"
    );
    assert!(
        closed.is_empty(),
        "closing a window the tiler never managed reset its corners: {closed:?}"
    );
}

#[test]
fn a_window_minimized_to_the_tray_gets_the_system_corners_back() {
    // Arrange -- minimizing takes the window out of the layout, but it
    // keeps the corners the tiler gave it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(2) });

    // Act -- the application hides it, as it does to go to the tray.
    h.window(2).visible = false;
    let effects = h.event(WindowEvent::Hidden { window: wid(2) }).to_vec();

    // Assert
    assert_eq!(effects, [Effect::Undecorate { window: wid(2) }]);
}
