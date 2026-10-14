//! Which windows the tiler remembers giving its corners to, beyond the
//! ones that hide or close: windows that leave without saying so, and
//! windows it manages without having decorated them.

use crate::config::WindowRule;

use super::*;

/// Opens a window the tiler only adopts when the user drags it, which
/// does not give it the configured corners.
fn adopt_by_drag(h: &mut Harness, id: u64, class: &str) {
    let mut window = FakeWindow::app(0);
    if let Some(facts) = window.facts.as_mut() {
        facts.class = class.into();
    }
    h.desktop.windows.insert(wid(id), window);
    h.event(WindowEvent::Moved { window: wid(id) });
}

#[test]
fn a_window_that_closed_without_a_report_leaves_no_corners_to_its_id() {
    // Arrange -- window 2 closes without a report, and a retile drops
    // it. A menu is then given the same id.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.desktop.windows.remove(&wid(2));
    h.action(Action::Retile);
    let mut menu = FakeWindow::app(0);
    menu.facts = None;
    menu.visible = false;
    h.desktop.windows.insert(wid(2), menu);

    // Act
    let effects = h.event(WindowEvent::Hidden { window: wid(2) }).to_vec();

    // Assert
    assert!(
        effects.is_empty(),
        "the menu was taken for the window that closed: {effects:?}"
    );
}

#[test]
fn a_window_minimized_without_a_report_still_gets_the_system_corners_back() {
    // Arrange -- a retile drops a window minimized without a report
    // from the layout, but it keeps the corners the tiler gave it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).minimized = true;
    h.action(Action::Retile);

    // Act -- the application hides it, as it does to go to the tray.
    h.window(2).visible = false;
    let effects = h.event(WindowEvent::Hidden { window: wid(2) }).to_vec();

    // Assert
    assert_eq!(effects, [Effect::Undecorate { window: wid(2) }]);
}

#[test]
fn shutdown_leaves_the_corners_of_a_window_never_decorated_alone() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    adopt_by_drag(&mut h, 2, "App");
    assert_eq!(h.active(0), vec![1, 2]);

    // Act
    let effects = h.step(Input::Shutdown).to_vec();

    // Assert -- both are shown; only the decorated one gets the
    // system's corners back.
    assert!(effects.contains(&Effect::Reveal { window: wid(2) }));
    assert!(effects.contains(&Effect::Undecorate { window: wid(1) }));
    assert!(
        !effects.contains(&Effect::Undecorate { window: wid(2) }),
        "{effects:?}"
    );
}

#[test]
fn a_rule_reload_leaves_the_corners_of_a_window_never_decorated_alone() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    adopt_by_drag(&mut h, 2, "Dragged");
    let rule = WindowRule {
        match_class: Some("Dragged".into()),
        match_title: None,
        match_path: None,
        manage: false,
    };

    // Act
    let effects = h.step(Input::Rules(vec![rule])).to_vec();

    // Assert -- it is let go, and its corners were never the tiler's.
    assert_eq!(h.active(0), vec![1]);
    assert!(
        !effects.contains(&Effect::Undecorate { window: wid(2) }),
        "{effects:?}"
    );
}
