//! The tiler, the coalescer and the echo filter together, the way the
//! platform runs them.

use crate::Direction;

use super::*;

/// Windows whose border overlay is showing on the fake desktop.
fn showing_borders(h: &Harness) -> Vec<u64> {
    let mut showing: Vec<u64> = h
        .desktop
        .borders
        .iter()
        .filter(|&(_, &showing)| showing)
        .map(|(w, _)| w.raw())
        .collect();
    showing.sort_unstable();
    showing
}

#[test]
fn a_move_lands_with_its_borders_once_input_goes_quiet() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]).through_platform();
    let left = h.rect_of(1);

    // Act
    h.action(Action::Move(Direction::Right));
    h.flush();

    // Assert -- the near miss of #71: a move that stays on screen.
    assert_eq!(h.rect_of(2), left);
    assert_eq!(showing_borders(&h), vec![1, 2]);
}

#[test]
fn switching_workspace_right_after_a_move_leaves_no_border_behind() {
    // Arrange -- #71: the move is held until input goes quiet, and the
    // switch hides both windows before that happens.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]).through_platform();

    // Act
    h.action(Action::Move(Direction::Right));
    h.action(Action::GoToWorkspace(2));
    h.flush();

    // Assert -- nothing is on screen, so no overlay may exist.
    assert!(
        h.desktop.borders.is_empty(),
        "border overlays left for hidden windows: {:?}",
        h.desktop.borders
    );
}

#[test]
fn closing_a_window_right_after_a_move_leaves_no_border_behind() {
    // Arrange -- #71: the window is destroyed before the held move lands.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]).through_platform();

    // Act
    h.action(Action::Move(Direction::Right));
    h.desktop.windows.remove(&wid(1));
    h.event(WindowEvent::Destroyed { window: wid(1) });
    h.flush();

    // Assert
    assert!(
        !h.desktop.borders.contains_key(&wid(1)),
        "border overlay left for a destroyed window: {:?}",
        h.desktop.borders
    );
    assert_eq!(showing_borders(&h), vec![2]);
}
