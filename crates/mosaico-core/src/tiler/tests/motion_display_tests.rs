//! Motions a display change ends.

use crate::tiler::Display;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::{MON_H, MON_W, displays};
use crate::{Action, Direction, Rect};

use super::{animated, placements, settle, swap, two};

/// One monitor whose work area loses 100 pixels at the bottom.
fn shorter() -> Vec<Display> {
    let mut monitors = displays(1);
    monitors[0].work_area = Rect::new(0, 0, MON_W, MON_H - 100);
    monitors
}

/// Whether `rect` is in the work area of [`shorter`].
fn fits(rect: &Rect) -> bool {
    rect.y + rect.height <= MON_H - 100
}

#[test]
fn a_display_change_ends_every_motion_in_the_slot_it_lays_the_window_out_in() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    platform.take_calls();

    // Act
    manager.handle_display_change(shorter(), 0, &[]);

    // Assert -- each window placed once, at once, in the smaller work area,
    // and no border drawn around the slot it was going to.
    let calls = platform.take_calls();
    let placed = placements(&calls);
    assert_eq!(placed.len(), 2, "{placed:?}");
    assert!(placed.iter().all(|(_, rect)| fits(rect)), "{placed:?}");
    let drawn = |call: &Call| match call {
        Call::ShowBorder(_, style) => Some(style.rect),
        _ => None,
    };
    assert!(
        calls.iter().filter_map(drawn).all(|r| fits(&r)),
        "{calls:?}"
    );
}

#[test]
fn a_display_change_stops_every_motion_where_it_is_until_it_is_rebuilt_for() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let start = [1, 2].map(|id| platform.window(id).rect);
    swap(&mut manager);
    platform.take_calls();

    // Act -- noted at once, rebuilt for once it has settled.
    manager.display_changing();
    let frames = settle(&platform, &mut manager);
    let meanwhile = platform.take_calls();
    let held = [1, 2].map(|id| platform.window(id).rect);
    manager.handle_display_change(shorter(), 0, &[]);

    // Assert -- nothing places the windows stopped while the change
    // settles, and the rebuild places each once, in the smaller work area.
    assert_eq!(frames, 0);
    assert!(placements(&meanwhile).is_empty(), "{meanwhile:?}");
    assert_eq!(held, start);
    let placed = placements(&platform.take_calls());
    assert_eq!(placed.len(), 2, "{placed:?}");
    assert!(placed.iter().all(|(_, rect)| fits(rect)), "{placed:?}");
}

#[test]
fn the_windows_a_display_change_stopped_go_to_their_targets_rebuilt_for_or_not() {
    for rebuilt in [true, false] {
        // Arrange -- a burst of changes that settles back where it started,
        // or monitors that cannot be read once it settles.
        let (platform, mut manager) = two(&animated());
        let (left, right) = (platform.window(1).rect, platform.window(2).rect);
        swap(&mut manager);
        manager.display_changing();
        platform.take_calls();

        // Act
        if rebuilt {
            manager.handle_display_change(displays(1), 0, &[]);
        } else {
            manager.display_change_skipped();
        }

        // Assert -- each at its target, as the swap left them.
        let placed = placements(&platform.take_calls());
        assert_eq!(placed, [(2, left), (1, right)], "rebuilt: {rebuilt}");
    }
}

#[test]
fn a_swap_released_while_a_display_change_settles_is_placed_at_once() {
    // Arrange -- the swap is held as the change is noted.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    manager.handle_action(&Action::Move(Direction::Right));
    manager.display_changing();
    platform.take_calls();

    // Act
    manager.flush_pending();

    // Assert -- nothing moves for a layout about to be replaced.
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
    assert!(!manager.wants_frames());
}

#[test]
fn a_window_placed_while_a_display_change_settles_stays_where_it_was_placed() {
    // Arrange -- the swap is stopped by the change, then undone.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    manager.display_changing();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    platform.take_calls();

    // Act
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- not sent back to where the swap was taking it.
    assert!(placements(&platform.take_calls()).is_empty());
}
