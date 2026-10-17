//! A drag the tiler follows and the bars: what the pointer does on a bar
//! meanwhile, and whether the bars let it through.

use crate::action::Origin;
use crate::event::DragStart;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, WindowEvent};

use super::support::{grab, places};

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
