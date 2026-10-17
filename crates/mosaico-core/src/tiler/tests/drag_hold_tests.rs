//! The window the user holds, with transitions on: whatever they do with
//! the keyboard meanwhile, nothing places or animates it until they let
//! it go.

use crate::id::wid;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{animated, settle, start};
use crate::{Action, Direction, WindowEvent};

/// Whether `calls` move window `id`, by a placement or a frame.
fn moves(calls: &[Call], id: u64) -> bool {
    calls.iter().any(|call| match call {
        Call::Place(window, _) => *window == wid(id),
        Call::PlaceFrames(frames) => frames.iter().any(|f| f.window == wid(id)),
        _ => false,
    })
}

#[test]
fn a_key_pressed_while_a_window_is_held_neither_places_nor_animates_it() {
    // Arrange -- the user drags window 1, which has focus, 300 pixels.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    platform.edit(1, |w| w.rect.x += 300);
    let held = platform.window(1).rect;
    platform.take_calls();

    // Act -- a key swaps it with window 2, which slides into its slot.
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert
    let calls = platform.take_calls();
    assert!(moves(&calls, 2), "{calls:?}");
    assert!(!moves(&calls, 1), "{calls:?}");
    assert_eq!(platform.window(1).rect, held);
}

#[test]
fn a_window_held_through_a_key_slides_to_its_slot_once_let_go() {
    // Arrange -- the same swap while the user holds window 1.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    let right = platform.window(2).rect;
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    platform.edit(1, |w| w.rect.x += 300);
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Act
    manager.handle_event(&WindowEvent::moved(wid(1)));
    let moving = manager.wants_frames();
    settle(&platform, &mut manager);

    // Assert -- to the slot the swap gave it.
    assert!(moving);
    assert_eq!(platform.window(1).rect, right);
}
