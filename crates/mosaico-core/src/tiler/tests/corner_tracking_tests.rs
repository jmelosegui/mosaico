//! Which windows the tiler remembers giving its corners to, as windows
//! leave without saying so.

use super::*;

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
