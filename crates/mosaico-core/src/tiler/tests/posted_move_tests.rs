//! The placement that follows a move made without waiting (#28).
//!
//! Such a move can land before the layout pass reaches its window, and a
//! placement that finds its window in place is skipped, repaint and all.
//! A window moved that way while it is on screen is still placed and
//! repainted in full, as it was before the move came first.

use crate::config::Config;

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
use super::*;

/// What the platform was asked to do to where `window` is and how it
/// draws, in order.
fn moves_of(calls: &[Call], window: u64) -> Vec<Call> {
    calls
        .iter()
        .filter(|c| {
            matches!(c, Call::Place(w, _) | Call::PrePlace(w, _) | Call::Repaint(w)
                if *w == wid(window))
        })
        .cloned()
        .collect()
}

#[test]
fn a_window_created_visible_is_placed_and_repainted_after_its_move_lands() {
    // Arrange -- moves that do not wait land at once on this platform
    let platform = FakePlatform::new(1, &[(1, 0), (99, 0)]);
    platform.edit(99, |w| w.visible = false);
    platform.land_pre_placements();
    let mut manager = start(&platform, None, &Config::default());
    platform.edit(99, |w| w.visible = true);
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::Created { window: wid(99) });

    // Assert
    let slot = platform.window(99).rect;
    assert_eq!(
        moves_of(&platform.take_calls(), 99),
        [
            Call::PrePlace(wid(99), slot),
            Call::Place(wid(99), slot),
            Call::Repaint(wid(99))
        ]
    );
}

#[test]
fn a_window_restored_into_another_slot_is_placed_and_repainted_after_its_move_lands() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.land_pre_placements();
    let mut manager = start(&platform, None, &Config::default());
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    platform.edit(2, |w| w.minimized = false);
    let before = platform.window(2).rect;
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::Restored { window: wid(2) });

    // Assert
    let slot = platform.window(2).rect;
    assert_ne!(slot, before, "restored into the same slot");
    assert_eq!(
        moves_of(&platform.take_calls(), 2),
        [
            Call::PrePlace(wid(2), slot),
            Call::Place(wid(2), slot),
            Call::Repaint(wid(2))
        ]
    );
}

#[test]
fn a_window_pre_placed_while_hidden_is_left_alone_once_in_its_slot() {
    // Arrange -- created hidden, and its move landed before it was shown
    let platform = FakePlatform::new(1, &[(1, 0), (99, 0)]);
    platform.edit(99, |w| w.visible = false);
    platform.land_pre_placements();
    let mut manager = start(&platform, None, &Config::default());
    manager.handle_event(&WindowEvent::Created { window: wid(99) });
    platform.edit(99, |w| w.visible = true);
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::Created { window: wid(99) });

    // Assert -- as before: its placement finds it in place
    assert_eq!(moves_of(&platform.take_calls(), 99), []);
}

#[test]
fn the_move_that_does_not_wait_comes_before_the_corners() {
    // Arrange -- setting corners makes no promise about waiting, so it
    // must not hold up the move
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert
    assert!(
        matches!(effects.first(), Some(Effect::PrePlace { window, .. }) if *window == wid(99)),
        "{effects:?}"
    );
}
