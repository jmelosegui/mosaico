//! What taking a window in must keep doing as before, next to the early
//! move of #28: windows the layout moves first anyway, windows already
//! in their slot, maximized windows, and steps that take nothing in.

use std::time::Instant;

use super::adopt_order_support::{moves, posted_move};
use super::*;

/// Whether every border a step draws comes after the last placement of
/// its window in that step.
fn borders_follow_placements(effects: &[Effect]) -> bool {
    effects.iter().enumerate().all(|(i, e)| match *e {
        Effect::Border { window, .. } => !effects[i..]
            .iter()
            .any(|later| matches!(*later, Effect::Place { window: w, .. } if w == window)),
        _ => true,
    })
}

#[test]
fn a_window_created_visible_on_an_empty_monitor_is_only_laid_out() {
    // Arrange
    let mut h = Harness::new(1, &[]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert -- the layout moves it first anyway
    assert_eq!(moves(&effects), ["decorate 99", "place 99", "focus 99"]);
}

#[test]
fn a_window_created_visible_on_a_monocle_workspace_is_only_laid_out() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::ToggleMonocle);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert -- the monocle window is the only one the layout moves
    assert_eq!(moves(&effects), ["decorate 99", "place 99", "focus 99"]);
}

#[test]
fn a_window_pre_placed_before_it_was_shown_is_taken_in_as_before() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut hidden = FakeWindow::app(0);
    hidden.visible = false;
    h.desktop.windows.insert(wid(99), hidden);

    // Act -- created hidden, then shown once the pre placement landed
    let created = h.event(WindowEvent::Created { window: wid(99) }).to_vec();
    let slot = posted_move(&created, 99).expect("pre placed");
    h.window(99).rect = slot;
    h.window(99).visible = true;
    let shown = h.event(WindowEvent::Created { window: wid(99) }).to_vec();

    // Assert -- in its slot already, so nothing is moved early
    assert_eq!(moves(&created), ["post move 99"]);
    assert_eq!(
        moves(&shown),
        ["decorate 99", "place 1", "place 99", "focus 99"]
    );
    assert_eq!(h.rect_of(99), slot);
}

#[test]
fn a_window_taken_in_while_maximized_is_laid_out_as_before() {
    // Arrange -- only a placement takes a window out of maximized state
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut maximized = FakeWindow::app(0);
    maximized.maximized = true;
    maximized.rect = Rect::new(0, 0, MON_W, MON_H);
    h.desktop.windows.insert(wid(99), maximized);

    // Act
    let effects = h.event(WindowEvent::Created { window: wid(99) }).to_vec();

    // Assert
    assert_eq!(
        moves(&effects),
        ["decorate 99", "place 1", "place 99", "focus 99"]
    );
}

#[test]
fn a_move_that_does_not_wait_leaves_the_planned_window_as_it_is() {
    // Arrange -- it may land later or not at all
    let mut desktop = FakeDesktop::default();
    let mut window = FakeWindow::app(0);
    window.maximized = true;
    desktop.windows.insert(wid(99), window.clone());
    let mut plan = Plan::new(&desktop, Instant::now());

    // Act
    plan.emit(Effect::PrePlace {
        window: wid(99),
        rect: Rect::new(964, 8, 948, 1064),
    });

    // Assert
    assert!(plan.is_maximized(wid(99)));
    assert!(!plan.is_minimized(wid(99)));
    assert_eq!(plan.rect(wid(99)), Some(window.rect));
}

#[test]
fn a_retile_still_moves_the_windows_in_layout_order() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.action(Action::Retile).to_vec();

    // Assert
    assert_eq!(moves(&effects), ["place 1", "place 2"]);
    assert!(borders_follow_placements(&effects));
}

#[test]
fn a_workspace_switch_keeps_the_order_of_its_effects() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    let away = h.action(Action::GoToWorkspace(2)).to_vec();
    let back = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert_eq!(
        moves(&away),
        ["drop border 1", "drop border 2", "hide 1", "hide 2"]
    );
    assert_eq!(
        moves(&back),
        ["show 1", "show 2", "place 1", "place 2", "focus 1"]
    );
    assert!(borders_follow_placements(&back));
}
