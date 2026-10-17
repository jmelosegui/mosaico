//! The space the tiler keeps free for the bars, from what the manager
//! reports, passed on the way the daemon does.

use std::time::Instant;

use super::tests::{bar, left, right, start};
use crate::bar::BarManager;
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::config::BarConfig;
use crate::tiler::{BarReservation, Display, Input, Settings, Tiler};
use crate::{MonitorId, MonitorKey, Rect};

#[path = "no_windows.rs"]
mod no_windows;

use no_windows::NoWindows;

/// The tiling area of each monitor in `areas`, once the tiler reserves
/// the space `bars` reports, as the daemon passes it on.
fn tiling_areas(bars: &BarManager, areas: &[Rect]) -> Vec<Rect> {
    let displays: Vec<Display> = areas
        .iter()
        .enumerate()
        .map(|(i, &work_area)| Display {
            id: MonitorId::from_raw(i as u64),
            key: MonitorKey::default(),
            work_area,
            primary: i == 0,
        })
        .collect();
    let now = Instant::now();
    let (mut tiler, _) = Tiler::new(Settings::default(), &displays, &[], &NoWindows, now);
    let reservation = BarReservation {
        height: bars.bar_height(),
        monitors: bars.bar_monitor_indices().to_vec(),
    };
    tiler.step(Input::Bar(reservation), &NoWindows, now);
    tiler.monitors().iter().map(|m| m.work_area()).collect()
}

/// `area` with the default bar height taken off the top.
fn below_bar(area: Rect) -> Rect {
    let height = BarConfig::default().height;
    Rect::new(area.x, area.y + height, area.width, area.height - height)
}

#[test]
fn no_space_is_reserved_when_no_configured_monitor_is_connected() {
    // Arrange -- #91: the bar is meant for a second monitor, and only
    // one is connected.
    let areas = [left()];
    let bars = start(
        bar("monitors = [1]"),
        &areas,
        &FakeRenderer::default(),
        &FakeSystem::default(),
    );

    // Act
    let tiling = tiling_areas(&bars, &areas);

    // Assert
    assert_eq!(tiling, areas);
}

#[test]
fn no_space_is_reserved_when_no_bar_can_be_made() {
    // Arrange -- #91
    let areas = [left(), right()];
    let renderer = FakeRenderer::default();
    renderer.fail_on(left());
    renderer.fail_on(right());
    let bars = start(
        BarConfig::default(),
        &areas,
        &renderer,
        &FakeSystem::default(),
    );

    // Act
    let tiling = tiling_areas(&bars, &areas);

    // Assert
    assert_eq!(tiling, areas);
}

#[test]
fn a_bar_on_every_monitor_reserves_space_on_every_monitor() {
    // Arrange -- an empty list means every monitor.
    let areas = [left(), right()];
    let bars = start(
        bar("monitors = []"),
        &areas,
        &FakeRenderer::default(),
        &FakeSystem::default(),
    );

    // Act
    let tiling = tiling_areas(&bars, &areas);

    // Assert
    assert_eq!(tiling, [below_bar(left()), below_bar(right())]);
}

#[test]
fn only_the_monitors_with_a_bar_reserve_space() {
    // Arrange
    let areas = [left(), right()];
    let bars = start(
        bar("monitors = [1]"),
        &areas,
        &FakeRenderer::default(),
        &FakeSystem::default(),
    );

    // Act
    let tiling = tiling_areas(&bars, &areas);

    // Assert
    assert_eq!(tiling, [left(), below_bar(right())]);
}
