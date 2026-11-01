//! What a resize leaves alone: the held edges it did not move, which keep
//! their splits as they were, and a window gone without a report, which
//! leaves the layout before the splits are worked out.

use crate::config::Config;
use crate::id::wid;
use crate::tiler::Input;
use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect, WindowEvent};

use super::support::{RIGHT, grab_to, held, holding, resize_to, resizing, unmoved};

/// Windows 1, 2 and 3 tiled in BSP, with splits moved by the edges of the
/// windows the user resizes.
fn three() -> Harness {
    Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0), (3, 0)])
}

/// The ratio the user gave split `split` of the BSP chain of `h`.
fn split(h: &Harness, split: usize) -> Option<f64> {
    let ratios = h.workspace(0, 0).split_ratios();
    ratios.ratio(LayoutKind::Bsp, split)
}

#[test]
fn a_corner_dragged_along_one_edge_leaves_the_other_split_to_the_config() {
    // Arrange -- window 3, the bottom of the right half, held by its top
    // left corner and dragged 150 pixels left only.
    let mut h = three();
    let bottom = h.rect_of(3);
    let wider = Rect::new(bottom.x - 150, bottom.y, bottom.width + 150, bottom.height);
    resize_to(&mut h, 3, held(true, true, false, false), wider);

    // Act -- a config with another ratio.
    let mut config = Config::with_gap(8);
    config.layout.ratio = 0.25;
    h.step(Input::Config(Box::new(config)));

    // Assert -- split 1 follows it: window 2 has a quarter of the height.
    assert!(split(&h, 0).is_some());
    assert_eq!(split(&h, 1), None);
    assert_eq!(h.rect_of(2).height, 262);
}

#[test]
fn an_edge_that_wobbled_a_pixel_keeps_its_split_as_it_was() {
    // Arrange -- window 2, the top of the right half, held by its bottom
    // left corner.
    let mut h = three();
    let top = h.rect_of(2);

    // Act -- its left edge 200 pixels left, its bottom edge one down.
    let moved = Rect::new(top.x - 200, top.y, top.width + 200, top.height + 1);
    resize_to(&mut h, 2, held(true, false, false, true), moved);

    // Assert
    assert!(split(&h, 0).is_some());
    assert_eq!(split(&h, 1), None);
}

#[test]
fn escape_on_a_window_wider_than_its_slot_moves_no_split() {
    // Arrange -- window 1, held 100 pixels wider than its slot, as by its
    // minimum size or on its way there, grabbed by its right edge and made
    // wider still.
    let mut h = Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0)]);
    let slot = h.rect_of(1);
    h.window(1).rect.width += 100;
    let start = h.rect_of(1);
    let wider = Rect {
        width: start.width + 200,
        ..start
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Act -- the system puts it back where it started, with the pointer
    // where the user pressed Escape.
    h.window(1).rect = start;
    let pointer = Some(holding(wider, RIGHT));
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert -- back in its slot.
    assert!(unmoved(&h));
    assert_eq!(h.rect_of(1), slot);
}

#[test]
fn a_window_gone_without_a_report_leaves_the_layout_before_a_resize_counts() {
    // Arrange -- window 1, the left half, is gone, and nothing said so.
    let mut h = three();
    h.window(1).visible = false;
    let top = h.rect_of(2);

    // Act -- the bottom edge of window 2, which borders split 1 while
    // window 1 is there, 100 pixels down.
    let taller = Rect {
        height: top.height + 100,
        ..top
    };
    resize_to(&mut h, 2, held(false, false, false, true), taller);

    // Assert -- window 2 is the left half now, whose bottom edge borders
    // no split, so it goes back to its slot there.
    assert_eq!(h.active(0), vec![2, 3]);
    assert!(unmoved(&h));
    assert_eq!(h.rect_of(2), Rect::new(8, 8, 948, 1064));
}

#[test]
fn a_corner_of_a_window_taller_than_its_slot_dragged_along_one_edge_moves_one_split() {
    // Arrange -- window 2, the top of the right half, held 60 pixels
    // taller than its slot, as by its minimum size.
    let mut h = three();
    h.window(2).rect.height += 60;
    let start = h.rect_of(2);

    // Act -- held by its bottom left corner, dragged 200 pixels left only.
    let wider = Rect::new(start.x - 200, start.y, start.width + 200, start.height);
    resize_to(&mut h, 2, held(true, false, false, true), wider);

    // Assert -- its bottom edge stayed where it started, below its slot,
    // and moves no split.
    assert!(split(&h, 0).is_some());
    assert_eq!(split(&h, 1), None);
}
