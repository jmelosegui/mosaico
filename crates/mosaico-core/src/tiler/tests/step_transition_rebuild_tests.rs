//! The steps that lay the windows out from scratch, and reloads.

use crate::Rect;
use crate::config::Config;
use crate::tiler::tests::{Harness, MON_H, MON_W, displays};
use crate::tiler::{BarReservation, Input};

use super::{Animate, Snap, animated, marks};

#[test]
fn the_start_and_the_bar_reservation_right_after_it_snap() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 1, &[(1, 0), (2, 0)]);
    let start = h.effects.clone();
    let bar = |height| {
        Input::Bar(BarReservation {
            height,
            monitors: Vec::new(),
        })
    };

    // Act
    let reserved = h.step(bar(30)).to_vec();
    let resized = h.step(bar(60)).to_vec();

    // Assert
    assert_eq!(marks(&start), [(1, Snap), (2, Snap)]);
    assert_eq!(marks(&reserved), [(1, Snap), (2, Snap)]);
    assert_eq!(marks(&resized), [(1, Animate), (2, Animate)]);
}

#[test]
fn a_display_change_snaps_every_window() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 2, &[(1, 0), (2, 1)]);
    let mut monitors = displays(2);
    monitors[1].work_area = Rect::new(MON_W, 0, MON_W, MON_H - 40);
    let bar = BarReservation::default();

    // Act
    let effects = h.step(Input::Displays { monitors, bar }).to_vec();

    // Assert
    assert_eq!(marks(&effects), [(1, Snap), (2, Snap)]);
}

#[test]
fn a_reload_lays_the_windows_out_with_the_transitions_it_brings() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let mut config = Config::with_gap(8);
    config.animations.enabled = true;
    config.layout.gap += 20;

    // Act
    let on = h.step(Input::Config(Box::new(config.clone()))).to_vec();
    config.animations.enabled = false;
    config.layout.gap += 20;
    let off = h.step(Input::Config(Box::new(config))).to_vec();

    // Assert
    assert_eq!(marks(&on), [(1, Animate), (2, Animate)]);
    assert_eq!(marks(&off), [(1, Snap), (2, Snap)]);
}
