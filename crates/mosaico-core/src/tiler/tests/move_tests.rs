use crate::Direction;

use super::*;

#[test]
fn moving_swaps_with_the_neighbor_and_may_be_coalesced() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert_eq!(h.active(0), vec![2, 1]);
    assert_eq!(h.rect_of(2), left);
    assert!(h.placements().iter().all(|p| p.2 == Timing::Coalesced));
    assert_eq!(
        h.tiler.focused_window(),
        Some(wid(1)),
        "focus stays on the moved window"
    );
}

#[test]
fn borders_after_a_move_use_the_new_slot() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);

    // Act
    let effects = h.action(Action::Move(Direction::Right)).to_vec();

    // Assert -- the tiling manager holds this border until the window
    // lands.
    let border_of_2 = effects.iter().rev().find_map(|e| match e {
        Effect::Border {
            window,
            style: Some(s),
        } if *window == wid(2) => Some(s.rect),
        _ => None,
    });
    assert_eq!(border_of_2, Some(left));
}

#[test]
fn moving_past_the_edge_sends_the_window_to_the_next_monitor() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);
    h.action(Action::Focus(Direction::Right));
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert -- it enters on the side it came from.
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.active(1), vec![2, 3]);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert!(h.rect_of(2).x >= MON_W);
}

#[test]
fn moving_onto_a_monocle_monitor_ends_monocle_there() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.action(Action::Focus(Direction::Right));
    h.action(Action::ToggleMonocle);
    h.action(Action::Focus(Direction::Left));

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert!(!h.workspace(1, 0).monocle());
    assert_eq!(h.active(1), vec![1, 2]);
}

#[test]
fn a_monocle_window_does_not_move() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::ToggleMonocle);

    // Act
    let effects = h.action(Action::Move(Direction::Right)).to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn the_cursor_follows_a_moved_window() {
    // Arrange
    let settings = Settings {
        mouse_follows_focus: true,
        ..Settings::with_gap(8)
    };
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.action(Action::Move(Direction::Right)).to_vec();

    // Assert
    assert_eq!(
        effects.last(),
        Some(&Effect::CenterCursor { window: wid(1) })
    );
}
