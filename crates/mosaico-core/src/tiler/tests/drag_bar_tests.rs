//! A drag the tiler follows and the bars: what the pointer does on a bar
//! meanwhile, a drop over a bar, and whether the bars let it through.

use crate::action::Origin;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W};
use crate::{Action, WindowEvent};

use super::support::{grab, places, reserve_bar, swapping};

/// The user drags window `id` by its title bar, held `from_left` pixels
/// from its left end, until the pointer is at `to`, and drops it there,
/// where the window is on the monitor that holds its center.
fn drop_held_at(h: &mut Harness, id: u64, from_left: i32, to: (i32, i32)) {
    let rect = h.rect_of(id);
    let held = (rect.x + from_left, rect.y + 10);
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some(held),
    };
    h.event(WindowEvent::MoveStarted {
        window: wid(id),
        start,
    });
    let window = h.window(id);
    (window.rect.x, window.rect.y) = (rect.x + to.0 - held.0, rect.y + to.1 - held.1);
    window.monitor = (window.rect.center_x() / MON_W) as usize;
    let pointer = Some(to);
    h.event(WindowEvent::Moved {
        window: wid(id),
        pointer,
    });
}

#[test]
fn a_click_on_the_layout_widget_of_its_bar_leaves_the_dragged_window_with_the_user() {
    // Arrange -- three windows on the monitor, and window 1 dragged.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    grab(&mut h, 1);

    // Act -- a click on the layout widget of the monitor's bar cycles its
    // layout, then window 1 is dropped.
    let cycled = h.action_from(Action::CycleLayout, Origin::Bar(0)).to_vec();
    let dragged = h.tiler.is_dragged(wid(1));
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- the others go to their new slots while it stays where the
    // user has it, and the drop puts it in its own.
    assert!(places(&cycled, 2) && places(&cycled, 3), "{cycled:?}");
    assert!(!places(&cycled, 1), "{cycled:?}");
    assert!(dragged);
    let slot = (h.tiler.compute_positions(0).into_iter())
        .find_map(|(window, rect)| (window == wid(1)).then_some(rect));
    assert!(places(&dropped, 1), "{dropped:?}");
    assert_eq!(Some(h.rect_of(1)), slot);
}

#[test]
fn a_window_dropped_over_its_own_monitors_bar_goes_back_to_its_slot() {
    // Arrange -- most of window 1 over the second monitor, which its rect
    // alone would have it join.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    reserve_bar(&mut h);
    let left = h.rect_of(1);

    // Act -- held near the left end of its title bar, it is dropped with
    // the pointer over the first monitor's bar, near its right edge.
    drop_held_at(&mut h, 1, 20, (MON_W - 20, 10));

    // Assert -- over no tile of its own monitor, as over a gap.
    assert_eq!((h.active(0), h.active(1)), (vec![1, 2], vec![3]));
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_window_dropped_over_another_monitors_bar_joins_the_end_of_its_workspace() {
    // Arrange -- most of window 1 still over the first monitor.
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    reserve_bar(&mut h);
    let width = h.rect_of(1).width;

    // Act -- held near the right end of its title bar, it is dropped with
    // the pointer over the second monitor's bar.
    drop_held_at(&mut h, 1, width - 20, (MON_W + 20, 10));

    // Assert -- on the monitor of that bar, before no tile.
    assert_eq!(h.active(0), vec![2]);
    assert_eq!(h.active(1), vec![3, 4, 1]);
}

#[test]
fn a_window_dragged_out_of_maximized_has_the_bars_drawn_again() {
    // Arrange -- window 1 is maximized, which the bars were told.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(1).maximized = true;
    h.event(WindowEvent::LocationChanged { window: wid(1) });
    h.tiler.take_bars_changed();
    let start = DragStart {
        maximized: Some(true),
        ..DragStart::default()
    };

    // Act -- the user grabs it, and the drag takes it out of maximized.
    h.event(WindowEvent::MoveStarted {
        window: wid(1),
        start,
    });
    h.window(1).maximized = false;
    h.event(WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- the bars no longer let the pointer through over it.
    assert!(h.tiler.is_dragged(wid(1)));
    assert!(h.tiler.take_bars_changed());
}
