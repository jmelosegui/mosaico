//! What the layout actions, and windows coming and going, do to the heights
//! of the stacks: a swap keeps every slot its size, cycling layouts,
//! monocle and a reload leave them, a retile forgets them, a window that
//! opens in a resized stack gets an equal share while the others shrink in
//! proportion. What windows coming and going do to the columns of the three
//! columns is in `split_column_tests.rs`.

use crate::config::Config;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{BarReservation, Input};
use crate::{Action, Direction, LayoutKind, WindowEvent};

use super::split_tests::rects;

/// Windows `1..=count` on one monitor, in `kind`.
pub(super) fn in_layout(kind: LayoutKind, count: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::new(1, &windows);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

/// Gives the column the window at `index` is in, on the first monitor of
/// `h`, the heights `shares`, and lays it out again, as the bar read again
/// does.
pub(super) fn resize(h: &mut Harness, index: usize, shares: &[f64]) {
    h.tiler.monitors[0]
        .active_ws_mut()
        .set_heights(index, shares);
    h.step(Input::Bar(BarReservation::default()));
}

/// The heights of the column the window at `index` of `kind` is in, on the
/// first monitor of `h`, to a millionth, and none for one nobody resized.
pub(super) fn heights(h: &Harness, kind: LayoutKind, index: usize) -> Vec<f64> {
    let shares = h.workspace(0, 0).split_ratios().heights(kind, index);
    let round = |share: f64| (share * 1e6).round() / 1e6;
    shares.unwrap_or_default().into_iter().map(round).collect()
}

#[test]
fn a_swap_down_the_stack_keeps_every_slot_its_size() {
    // Arrange -- window 2, first of the stack, has a quarter of it.
    let mut h = in_layout(LayoutKind::VerticalStack, 4);
    resize(&mut h, 1, &[0.25, 0.375, 0.375]);
    let (top, next) = (h.rect_of(2), h.rect_of(3));

    // Act -- window 2 moved down, past window 3.
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::Move(Direction::Down));

    // Assert -- the windows trade places.
    assert_eq!(h.active(0), vec![1, 3, 2, 4]);
    assert_eq!((h.rect_of(3), h.rect_of(2)), (top, next));
}

#[test]
fn the_heights_of_the_stacks_stay_through_cycling_layouts_monocle_and_a_reload() {
    // Arrange -- the stack resized, and the left column.
    let mut h = in_layout(LayoutKind::VerticalStack, 5);
    resize(&mut h, 1, &[0.4, 0.3, 0.2, 0.1]);
    h.action(Action::CycleLayout);
    resize(&mut h, 1, &[0.7, 0.3]);
    let mut config = Config::with_gap(8);
    config.layout.ratio = 0.4;

    // Act -- around to BSP, monocle on and off, the configured ratio
    // changed, and back to the stack.
    h.action(Action::CycleLayout);
    h.action(Action::ToggleMonocle);
    h.action(Action::ToggleMonocle);
    h.step(Input::Config(Box::new(config)));
    h.action(Action::CycleLayout);

    // Assert
    assert_eq!(
        heights(&h, LayoutKind::VerticalStack, 1),
        [0.4, 0.3, 0.2, 0.1]
    );
    assert_eq!(heights(&h, LayoutKind::ThreeColumn, 1), [0.7, 0.3]);
}

#[test]
fn a_window_that_opens_in_a_resized_stack_gets_an_equal_share_and_the_others_shrink() {
    // Arrange -- windows 2, 3 and 4 down the stack, window 2 with half.
    let mut h = in_layout(LayoutKind::VerticalStack, 4);
    resize(&mut h, 1, &[0.5, 0.25, 0.25]);

    // Act
    h.open(5, 0);

    // Assert -- a quarter of the 1040 pixels the four share, and three
    // quarters of what each had.
    assert_eq!(
        heights(&h, LayoutKind::VerticalStack, 1),
        [0.375, 0.1875, 0.1875, 0.25]
    );
    let rows = [2, 3, 4, 5].map(|id| (h.rect_of(id).y, h.rect_of(id).height));
    assert_eq!(rows, [(8, 390), (406, 195), (609, 195), (812, 260)]);
}

#[test]
fn a_retile_forgets_the_heights_of_the_stacks_too() {
    // Arrange -- the stack resized.
    let mut h = in_layout(LayoutKind::VerticalStack, 4);
    let configured = rects(&h, &[1, 2, 3, 4]);
    resize(&mut h, 1, &[0.25, 0.6, 0.15]);

    // Act
    h.action(Action::Retile);

    // Assert
    assert_eq!(rects(&h, &[1, 2, 3, 4]), configured);
    assert!(heights(&h, LayoutKind::VerticalStack, 1).is_empty());
}
