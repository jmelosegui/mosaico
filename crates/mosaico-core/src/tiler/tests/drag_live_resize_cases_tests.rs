//! The tile beside a held edge while it is dragged, in the cases that
//! move it no further: a report that moves nothing or no split, with
//! `resize_splits` off, in monocle, and when no edge can be read; and how
//! a layout of its monitor meanwhile is followed live again.

use crate::config::Config;
use crate::tiler::drag::live::support::{following, pair, report, widened};
use crate::tiler::drag::resize::support::{RIGHT, grab_by, grab_to, resizing};
use crate::tiler::{Input, Settings, Timing};
use crate::{Action, Rect};

#[test]
fn a_report_that_moves_nothing_places_nothing() {
    // Arrange
    let mut h = pair(resizing());
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- the same rect again, and then a few pixels off the start.
    let again = report(&mut h, widened(200));
    let wobble = report(&mut h, widened(3));
    let still = report(&mut h, widened(2));

    // Assert -- the wobble moves no split, so window 2 goes back once.
    assert!(again.is_empty(), "{again:?}");
    assert_eq!(wobble, vec![(2, Rect::new(964, 8, 948, 1064), Timing::Now)]);
    assert!(still.is_empty(), "{still:?}");
}

#[test]
fn with_resize_splits_off_nothing_follows_the_edge() {
    // Arrange
    let mut h = pair(Settings::default());

    // Act
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Assert
    assert!(h.placements().is_empty(), "{:?}", h.effects);
}

#[test]
fn in_monocle_nothing_follows_the_edge() {
    // Arrange -- window 1 fills the monitor.
    let mut h = pair(resizing());
    h.action(Action::ToggleMonocle);
    let front = h.rect_of(1);

    // Act -- its right edge 300 pixels left.
    let narrower = Rect {
        width: front.width - 300,
        ..front
    };
    grab_to(&mut h, 1, RIGHT, narrower);

    // Assert
    assert!(h.placements().is_empty(), "{:?}", h.effects);
}

#[test]
fn a_window_that_reports_no_grab_is_followed_by_the_edge_read_from_it() {
    // Arrange
    let mut h = pair(resizing());

    // Act
    grab_by(&mut h, 1, None, RIGHT, widened(200));

    // Assert
    assert_eq!(h.placements(), vec![(2, following(), Timing::Now)]);
}

#[test]
fn a_window_that_reports_no_grab_and_whose_edges_all_moved_is_followed_by_nothing() {
    // Arrange
    let mut h = pair(resizing());
    let left = h.rect_of(1);

    // Act -- both edges of each axis moved, as a move does.
    let elsewhere = Rect::new(left.x + 100, left.y + 50, left.width, left.height);
    grab_by(&mut h, 1, None, RIGHT, elsewhere);

    // Assert
    assert!(h.placements().is_empty(), "{:?}", h.effects);
}

#[test]
fn a_window_that_opens_mid_drag_is_laid_out_live_from_the_next_report() {
    // Arrange
    let mut h = pair(resizing());
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- window 3 opens, which lays the monitor out as it is, and the
    // edge goes on.
    h.open(3, 0);
    let placed = report(&mut h, widened(200));

    // Assert -- windows 2 and 3 share the right of the edge.
    let top = Rect::new(1164, 8, 748, 528);
    let bottom = Rect::new(1164, 544, 748, 528);
    assert_eq!(
        placed,
        vec![(2, top, Timing::Now), (3, bottom, Timing::Now)]
    );
}

#[test]
fn a_config_read_again_mid_drag_lays_the_tile_out_and_the_next_report_live_again() {
    // Arrange
    let mut h = pair(resizing());
    let right = h.rect_of(2);
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- read again, which lays the monitor out as its workspace has
    // it, and the same report.
    let mut config = Config::default();
    config.mouse.resize_splits = true;
    h.step(Input::Config(Box::new(config)));
    let laid_out = h.rect_of(2);
    let placed = report(&mut h, widened(200));

    // Assert
    assert_eq!(laid_out, right);
    assert_eq!(placed, vec![(2, following(), Timing::Now)]);
}
