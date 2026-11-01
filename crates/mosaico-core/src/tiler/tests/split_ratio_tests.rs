//! What the layout actions, and windows coming and going, do to the splits
//! the user moved: monocle leaves them, cycling layouts finds each layout's
//! own again, a swap keeps every slot its size, a window that opens gets a
//! split at the configured ratio and one that closes takes its split with
//! it. What a retile does is in `split_retile_tests.rs`.

use crate::config::Config;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{BarReservation, Input};
use crate::{Action, Direction, Rect, WindowEvent};

/// Moves the splits of the layout on screen on the first monitor of `h`
/// to `ratios`, in order, and lays it out again.
fn move_splits(h: &mut Harness, ratios: &[f64]) {
    move_splits_on(h, 0, ratios);
}

/// Moves the splits of the layout on screen on monitor `mon` of `h` to
/// `ratios`, in order, and lays the monitors out again, as the bar read
/// again does, which keeps them.
pub(super) fn move_splits_on(h: &mut Harness, mon: usize, ratios: &[f64]) {
    let ws = h.tiler.monitors[mon].active_ws_mut();
    for (split, &ratio) in ratios.iter().enumerate() {
        ws.set_split_ratio(split, ratio);
    }
    h.step(Input::Bar(BarReservation::default()));
}

/// Where windows `ids` of `h` are.
pub(super) fn rects(h: &Harness, ids: &[u64]) -> Vec<Rect> {
    ids.iter().map(|&id| h.rect_of(id)).collect()
}

#[test]
fn cycling_layouts_finds_each_layouts_own_splits_again() {
    // Arrange -- the first BSP split, then the stack's master, moved.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    move_splits(&mut h, &[0.75]);
    h.action(Action::CycleLayout);
    let stack_as_configured = h.rect_of(1).width;
    move_splits(&mut h, &[0.25]);

    // Act -- through the three columns, which nobody moved, and around.
    h.action(Action::CycleLayout);
    let columns = h.rect_of(1).width;
    h.action(Action::CycleLayout);
    let bsp = h.rect_of(1).width;
    h.action(Action::CycleLayout);
    let stack = h.rect_of(1).width;

    // Assert
    assert_eq!(stack_as_configured, 948);
    assert_eq!(columns, 952);
    assert_eq!(bsp, 1424);
    assert_eq!(stack, 472);
}

#[test]
fn monocle_on_and_off_leaves_the_splits_where_they_were() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    move_splits(&mut h, &[0.75, 0.25]);
    let moved = rects(&h, &[1, 2, 3]);

    // Act
    h.action(Action::ToggleMonocle);
    h.action(Action::ToggleMonocle);

    // Assert
    assert_eq!(rects(&h, &[1, 2, 3]), moved);
}

#[test]
fn a_swap_keeps_every_slot_its_size() {
    // Arrange -- window 1, focused, has three quarters of the width.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    move_splits(&mut h, &[0.75]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert -- the windows trade places.
    assert_eq!(h.active(0), vec![2, 1]);
    assert_eq!((h.rect_of(2), h.rect_of(1)), (left, right));
}

#[test]
fn a_window_that_opens_gets_a_split_at_the_configured_ratio() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    move_splits(&mut h, &[0.75]);

    // Act
    h.open(3, 0);

    // Assert -- window 1 keeps its width, and the rest is halved.
    let expected = [
        Rect::new(8, 8, 1424, 1064),
        Rect::new(1440, 8, 472, 528),
        Rect::new(1440, 544, 472, 528),
    ];
    assert_eq!(rects(&h, &[1, 2, 3]), expected);
}

#[test]
fn a_window_that_closes_takes_its_split_with_it() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    move_splits(&mut h, &[0.75, 0.25]);

    // Act
    h.desktop.windows.remove(&wid(2));
    h.event(WindowEvent::Destroyed { window: wid(2) });

    // Assert -- window 3 takes the rest whole, beside window 1.
    let expected = [Rect::new(8, 8, 1424, 1064), Rect::new(1440, 8, 472, 1064)];
    assert_eq!(rects(&h, &[1, 3]), expected);
}

#[test]
fn a_new_configured_ratio_moves_only_the_splits_the_user_left_alone() {
    // Arrange -- the first split moved, the second not.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    move_splits(&mut h, &[0.75]);
    let mut config = Config::with_gap(8);
    config.layout.ratio = 0.25;

    // Act
    h.step(Input::Config(Box::new(config)));

    // Assert
    assert_eq!(h.rect_of(1).width, 1424);
    assert_eq!(h.rect_of(2).height, 262);
}
