//! A window dragged over another monitor where a window is maximized or
//! fills the monitor, as a fullscreen one does: only the preview shows
//! there, and no window there is placed until a drop there, so dragging
//! across leaves it as it was (#160 review, round 1, finding 1).

use crate::Rect;
use crate::tiler::drag::live::support::{monocle_on_second, on_two};
use crate::tiler::drag::support::{center, drop_at, grab, report_at, swapping};
use crate::tiler::tests::{Harness, MON_H, MON_W};

/// The whole of the second monitor.
const WHOLE: Rect = Rect {
    x: MON_W,
    y: 0,
    width: MON_W,
    height: MON_H,
};

/// Window 1 dragged over window 3, back over its own slot, and dropped
/// there. Returns every placement that made.
fn across_and_home(h: &mut Harness) -> Vec<u64> {
    let (home, over) = (center(h.rect_of(1)), center(h.rect_of(3)));
    grab(h, 1);
    report_at(h, 1, over);
    let mut placed = h.placements();
    report_at(h, 1, home);
    placed.extend(h.placements());
    drop_at(h, 1, home);
    placed.extend(h.placements());
    placed.into_iter().map(|(id, ..)| id).collect()
}

#[test]
fn a_maximized_window_there_is_left_maximized() {
    // Arrange -- window 4 maximized over the second monitor.
    let mut h = on_two(swapping(), &[3, 4]);
    h.window(4).rect = WHOLE;
    h.window(4).maximized = true;
    let third = h.rect_of(3);

    // Act
    let placed = across_and_home(&mut h);

    // Assert -- no window there was placed, and window 4 is still
    // maximized where it was.
    assert!(placed.iter().all(|&id| id != 3 && id != 4), "{placed:?}");
    assert!(h.window(4).maximized);
    assert_eq!((h.rect_of(3), h.rect_of(4)), (third, WHOLE));
}

#[test]
fn a_window_that_fills_its_monitor_there_is_left_where_it_is() {
    // Arrange -- window 4 over the whole of the second monitor, as a
    // fullscreen window is, without being maximized.
    let mut h = on_two(swapping(), &[3, 4]);
    h.window(4).rect = WHOLE;

    // Act
    let placed = across_and_home(&mut h);

    // Assert
    assert!(placed.iter().all(|&id| id != 3 && id != 4), "{placed:?}");
    assert_eq!(h.rect_of(4), WHOLE);
}

#[test]
fn a_maximized_monocle_window_there_is_left_maximized() {
    // Arrange -- window 3 in monocle on the second monitor, maximized.
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    h.window(3).rect = WHOLE;
    h.window(3).maximized = true;

    // Act
    let placed = across_and_home(&mut h);

    // Assert
    assert!(placed.iter().all(|&id| id != 3 && id != 4), "{placed:?}");
    assert!(h.window(3).maximized);
    assert_eq!(h.rect_of(3), WHOLE);
}
