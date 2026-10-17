//! The preview of a window whose size changed on the way: a few pixels,
//! more on its own monitor, or scaled up across two monitors.

use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W};
use crate::{Rect, WindowEvent};

use super::super::support::{center, grab, report_at, shown, swapping};

#[test]
fn a_window_a_few_pixels_off_its_size_shows_the_slot_of_the_tile_under_the_pointer() {
    // Arrange -- a pixel wider, as a trip across a monitor of another
    // scale leaves it, and the drop swaps it all the same.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    h.window(1).rect.width += 1;

    // Act
    let effects = report_at(&mut h, 1, center(right));

    // Assert
    assert_eq!(shown(&effects), Some(right));
}

#[test]
fn a_window_whose_size_changed_more_on_its_monitor_shows_its_own_slot() {
    // Arrange -- dropped there, it would be taken for a snap, which goes
    // back to its slot.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    grab(&mut h, 1);
    h.window(1).rect.width += 40;

    // Act
    let effects = report_at(&mut h, 1, center(right));

    // Assert
    assert_eq!(shown(&effects), Some(left));
}

#[test]
fn a_window_rescaled_across_two_monitors_shows_the_slot_of_the_tile_under_the_pointer() {
    // Arrange -- grabbed near its right edge and dragged over window 2, by
    // the edge of the first monitor, until most of it is on the second,
    // where the system scales it up for a monitor of another scale.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    h.window(1).rect = Rect::new(MON_W - 300, right.y, 1422, 1000);
    h.window(1).monitor = 1;
    h.desktop.cursor = Some((MON_W - 20, right.center_y()));

    // Act
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert -- the drop would swap it with window 2, not snap it back.
    assert_eq!(shown(&effects), Some(right));
}
