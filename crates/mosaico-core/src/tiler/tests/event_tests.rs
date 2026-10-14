use super::*;

#[test]
fn a_new_window_joins_the_focused_monitor_and_takes_focus() {
    // Arrange -- the OS put the window on the other monitor.
    let mut h = Harness::new(2, &[(1, 0)]);

    // Act
    h.open(2, 1);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert!(h.active(1).is_empty());
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(h.focus_request(), Some(2));
    assert!(h.rect_of(2).x < MON_W, "moved to the focused monitor");
}

#[test]
fn a_repeated_show_for_a_managed_window_is_ignored() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.event(WindowEvent::Created { window: wid(1) }).to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn minimizing_removes_the_window_and_restoring_brings_it_back() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.window(2).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(2) });
    let after_minimize = h.active(0);
    h.window(2).minimized = false;
    h.event(WindowEvent::Restored { window: wid(2) });

    // Assert
    assert_eq!(after_minimize, vec![1]);
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
}

#[test]
fn minimize_action_asks_the_platform_and_waits_for_the_report() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.action(Action::MinimizeFocused).to_vec();

    // Assert
    assert_eq!(effects, vec![Effect::Minimize { window: wid(1) }]);
    assert_eq!(
        h.active(0),
        vec![1],
        "removed when the minimize is reported"
    );
}

#[test]
fn a_hide_report_for_a_window_still_visible_is_ignored() {
    // Arrange -- some applications report a hide while repainting.
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    h.event(WindowEvent::Hidden { window: wid(1) });

    // Assert
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn a_real_hide_removes_the_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).visible = false;

    // Act
    h.event(WindowEvent::Hidden { window: wid(2) });

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert!(h.effects.contains(&Effect::Undecorate { window: wid(2) }));
}

#[test]
fn a_hide_and_show_while_an_app_replaces_its_surface_change_nothing() {
    // Arrange -- what a GPU-accelerated application reports while it
    // replaces the surface it draws on (the Windows translation test of
    // the same case produces these two). The window stays on screen.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let layout = (h.rect_of(1), h.rect_of(2));

    // Act
    let hidden = h.event(WindowEvent::Hidden { window: wid(1) }).to_vec();
    let shown = h.event(WindowEvent::Created { window: wid(1) }).to_vec();

    // Assert -- no effect at all, so it keeps its slot, border and focus.
    assert_eq!(hidden, vec![]);
    assert_eq!(shown, vec![]);
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!((h.rect_of(1), h.rect_of(2)), layout);
}

#[test]
fn clicking_a_window_on_another_monitor_moves_focus_there() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.event(WindowEvent::Focused { window: wid(2) });

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(h.focus_request(), None, "the OS already moved focus");
}

#[test]
fn a_focused_dialog_puts_the_border_on_its_owner() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    let mut dialog = FakeWindow::app(1);
    dialog.facts = None;
    dialog.owner = Some(wid(2));
    h.desktop.windows.insert(wid(9), dialog);

    // Act
    h.event(WindowEvent::Focused { window: wid(9) });

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert!(h.active(1) == vec![2], "the dialog is not tiled");
}

#[test]
fn focusing_an_unknown_window_adopts_it_where_it_is() {
    // Arrange -- some applications never report being created.
    let mut h = Harness::new(2, &[(1, 0)]);
    h.desktop.windows.insert(wid(2), FakeWindow::app(1));

    // Act
    h.event(WindowEvent::Focused { window: wid(2) });

    // Assert
    assert_eq!(h.active(1), vec![2]);
}

#[test]
fn a_title_change_adopts_a_window_missed_earlier() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.desktop.windows.insert(wid(2), FakeWindow::app(0));

    // Act
    h.event(WindowEvent::TitleChanged { window: wid(2) });

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn dragging_a_window_to_another_monitor_moves_it_there() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 0)]);

    // Act
    h.window(2).monitor = 1;
    h.event(WindowEvent::Moved { window: wid(2) });

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.active(1), vec![2]);
    assert!(h.rect_of(2).x >= MON_W);
}

#[test]
fn maximizing_the_focused_window_hides_its_border_and_keeps_it_maximized() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.window(1).maximized = true;
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Place { .. })),
        "a maximized window is not snapped back"
    );
    assert!(effects.contains(&Effect::Border {
        window: wid(1),
        style: None
    }));
}

#[test]
fn restoring_the_focused_window_from_maximized_snaps_it_back() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(1).maximized = true;
    h.event(WindowEvent::LocationChanged { window: wid(1) });

    // Act
    h.window(1).maximized = false;
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Place { window, .. } if *window == wid(1)))
    );
}

#[test]
fn hovering_a_window_focuses_it_without_moving_the_cursor() {
    // Arrange
    let settings = Settings {
        mouse_follows_focus: true,
        ..Settings::default()
    };
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.event(WindowEvent::MouseHover { window: wid(2) }).to_vec();

    // Assert
    assert!(effects.contains(&Effect::Focus {
        window: wid(2),
        move_cursor: false
    }));
}
