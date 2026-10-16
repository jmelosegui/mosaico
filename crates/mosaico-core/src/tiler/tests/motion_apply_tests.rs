//! Which placements become motions as they are applied.

use crate::config::Config;
use crate::{Action, Rect};

use super::{animated, placements, settle, swap, two};

#[test]
fn a_swap_that_may_animate_is_left_to_the_frame_step() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);

    // Act
    swap(&mut manager);
    let at_once = placements(&platform.take_calls());
    let moving = manager.wants_frames();
    let end = manager.frame_due(None);
    let stepped = manager.frame_due(end);
    settle(&platform, &mut manager);

    // Assert -- each ends exactly in its slot, with a full placement. A
    // frame step taken at the end of the motions leaves nothing due.
    assert!(at_once.is_empty(), "{at_once:?}");
    assert!(moving);
    assert!(end.is_some());
    assert_eq!(stepped, None);
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
    assert!(!manager.wants_frames());
    assert_eq!(manager.frame_due(None), None);
}

#[test]
fn with_animations_off_a_swap_is_placed_at_once() {
    // Arrange
    let (platform, mut manager) = two(&Config::default());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);

    // Act
    swap(&mut manager);

    // Assert
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
    assert!(!manager.wants_frames());
}

#[test]
fn nothing_animates_while_the_system_turns_animations_off() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let mut ignoring = animated();
    ignoring.animations.follow_system = false;
    let (other, mut ignores) = two(&ignoring);
    platform.allow_animations(false);
    other.allow_animations(false);

    // Act
    swap(&mut manager);
    swap(&mut ignores);

    // Assert -- unless the config does not follow the system.
    assert_eq!(placements(&platform.take_calls()).len(), 2);
    assert!(!manager.wants_frames());
    assert!(placements(&other.take_calls()).is_empty());
    assert!(ignores.wants_frames());
}

#[test]
fn a_maximized_window_is_placed_at_once_while_its_neighbour_animates() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let right = platform.window(2).rect;
    platform.edit(1, |w| w.maximized = true);

    // Act
    swap(&mut manager);

    // Assert -- placing it is what takes it out of maximized.
    assert_eq!(placements(&platform.take_calls()), [(1, right)]);
    assert!(!platform.window(1).maximized);
    assert!(manager.wants_frames());
}

#[test]
fn a_window_that_hardly_moves_is_placed_at_once() {
    // Arrange -- the bar takes 4 pixels: each window moves its top edge
    // down by that much and no further.
    let (platform, mut manager) = two(&animated());
    manager.refresh_border();
    let before = [1, 2].map(|id| platform.window(id).rect);

    // Act
    manager.adjust_work_areas_for_bar(4, &[]);

    // Assert
    let lower = |r: Rect| Rect::new(r.x, r.y + 4, r.width, r.height - 4);
    let placed = placements(&platform.take_calls());
    assert_eq!(placed, [(1, lower(before[0])), (2, lower(before[1]))]);
    assert!(!manager.wants_frames());
}

#[test]
fn a_placement_made_at_once_ends_the_motion_of_its_window() {
    // Arrange -- a swap in flight, then the system turns animations off.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    platform.allow_animations(false);

    // Act
    manager.handle_action(&Action::Retile);

    // Assert -- placed in full at once, and nothing is left moving.
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
    assert!(!manager.wants_frames());
    assert_eq!(settle(&platform, &mut manager), 0);
}

#[test]
fn a_manager_with_its_transitions_turned_off_places_every_window_at_once() {
    // Arrange -- as when the daemon cannot have display frames.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    manager.turn_off_transitions();

    // Act
    swap(&mut manager);

    // Assert
    let mut placed = placements(&platform.take_calls());
    placed.sort_by_key(|&(window, _)| window);
    assert_eq!(placed, [(1, right), (2, left)]);
    assert!(!manager.wants_frames());
}
