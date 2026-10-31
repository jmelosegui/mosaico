use super::*;

#[test]
fn startup_tiles_existing_windows_on_their_monitor() {
    // Arrange / Act
    let h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.active(1), vec![3]);
    assert!(h.rect_of(1).x < h.rect_of(2).x, "side by side");
    assert!(h.rect_of(3).x >= MON_W, "stays on the second monitor");
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn startup_skips_windows_the_platform_rules_out() {
    // Arrange
    let mut desktop_window = FakeWindow::app(0);
    desktop_window.facts = None;
    let mut h = Harness::new(1, &[(1, 0)]);
    h.desktop.windows.insert(wid(2), desktop_window);

    // Act
    h.event(WindowEvent::Created { window: wid(2) });

    // Assert
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn startup_decorates_and_borders_managed_windows() {
    // Arrange / Act
    let settings = Settings::default();
    let desktop = {
        let mut d = FakeDesktop::default();
        d.windows.insert(wid(1), FakeWindow::app(0));
        d
    };
    let (_, effects) = Tiler::new(
        settings,
        &displays(1),
        &[wid(1)],
        &desktop,
        std::time::Instant::now(),
    );

    // Assert
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Decorate { window, .. } if *window == wid(1)))
    );
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::Border {
            window,
            style: Some(_)
        } if *window == wid(1)
    )));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Focus { window, .. } if *window == wid(1)))
    );
}

#[test]
fn closing_a_window_reflows_its_neighbors() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.desktop.windows.remove(&wid(2));
    h.event(WindowEvent::Destroyed { window: wid(2) });

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.rect_of(1).width, MON_W - 16, "fills the monitor");
    assert!(h.effects.contains(&Effect::DropBorder { window: wid(2) }));
}

#[test]
fn closing_the_focused_window_promotes_a_sibling() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));

    // Act
    h.event(WindowEvent::Destroyed { window: wid(1) });

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn every_window_gets_a_placement_even_when_already_in_place() {
    // Arrange -- the platform skips the ones already there, against
    // the window's real position when it applies them.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.action(Action::Retile);

    // Assert
    let placed: Vec<u64> = h.placements().iter().map(|p| p.0).collect();
    assert_eq!(placed, vec![1, 2]);
    assert!(h.placements().iter().all(|p| p.2 == Timing::Now));
}

#[test]
fn layout_drops_windows_that_vanished_without_a_report() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).visible = false;

    // Act
    h.action(Action::Retile);

    // Assert
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn monocle_fills_the_work_area_with_the_focused_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.action(Action::ToggleMonocle);

    // Assert
    assert_eq!(h.rect_of(1), Rect::new(8, 8, MON_W - 16, MON_H - 16));
    let border_of_2 = h.effects.iter().rev().find_map(|e| match e {
        Effect::Border { window, style } if *window == wid(2) => Some(style.clone()),
        _ => None,
    });
    assert_eq!(
        border_of_2,
        Some(None),
        "only the monocle window has a border"
    );
}

#[test]
fn cycling_the_layout_changes_only_the_focused_workspace() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    let before = h.workspace(1, 0).layout_kind();

    // Act
    h.action(Action::CycleLayout);

    // Assert
    assert_ne!(h.workspace(0, 0).layout_kind(), before);
    assert_eq!(h.workspace(1, 0).layout_kind(), before);
}

#[test]
fn shutdown_reveals_every_window_and_restores_frames() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));

    // Act
    let effects = h.step(Input::Shutdown).to_vec();

    // Assert
    for w in [wid(1), wid(2)] {
        assert!(effects.contains(&Effect::Reveal { window: w }));
        assert!(effects.contains(&Effect::Undecorate { window: w }));
    }
    assert!(h.desktop.windows.values().all(|w| !w.cloaked));
}

#[test]
fn rule_reload_lets_go_of_windows_no_longer_managed() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    if let Some(facts) = h.window(2).facts.as_mut() {
        facts.class = "Excluded".into();
    }
    let rule = crate::config::WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        action: crate::config::RuleAction::Ignore,
    };

    // Act
    h.step(Input::Rules(vec![rule]));

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert!(h.effects.contains(&Effect::Undecorate { window: wid(2) }));
}
