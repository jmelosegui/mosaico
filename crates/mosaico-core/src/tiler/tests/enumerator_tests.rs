//! The order monitors are listed in, whatever order the platform finds
//! them in.

use crate::{Error, MonitorKey};

use super::*;

/// A platform that finds these monitors, in this order, or cannot list
/// monitors at all.
struct FakeEnumerator {
    found: Option<Vec<Display>>,
}

impl FakeEnumerator {
    fn finding(found: Vec<Display>) -> Self {
        Self { found: Some(found) }
    }
}

impl Enumerator for FakeEnumerator {
    fn windows(&self) -> Result<Vec<WindowId>> {
        Ok(Vec::new())
    }

    fn connected_monitors(&self) -> Result<Vec<Display>> {
        self.found
            .clone()
            .ok_or(Error::Unsupported("listing monitors"))
    }

    fn details(&self, _window: WindowId) -> WindowDetails {
        WindowDetails {
            facts: WindowFacts::default(),
            rect: None,
            minimized: false,
            maximized: false,
            monitor: MonitorId::from_raw(0),
            frame: None,
        }
    }
}

/// Monitor `id`, a 1920x1080 work area with its top left corner at
/// `(x, y)`. Monitor 0 is the primary one.
fn monitor(id: u64, x: i32, y: i32) -> Display {
    Display {
        id: MonitorId::from_raw(id),
        key: MonitorKey {
            stable_id: Some(format!("mon{id}")),
            ..Default::default()
        },
        work_area: Rect::new(x, y, 1920, 1080),
        primary: id == 0,
    }
}

/// The ids of `monitors`, in order.
fn ids(monitors: &[Display]) -> Vec<u64> {
    monitors.iter().map(|m| m.id.raw()).collect()
}

#[test]
fn monitors_are_listed_left_to_right() {
    // Arrange
    let left = monitor(2, -1920, 0);
    let middle = monitor(0, 0, 0);
    let right = monitor(1, 1920, 0);
    let platform = FakeEnumerator::finding(vec![right.clone(), left.clone(), middle.clone()]);

    // Act
    let monitors = platform.monitors().expect("the monitors can be listed");

    // Assert
    assert_eq!(monitors, vec![left, middle, right]);
}

#[test]
fn monitors_that_share_a_left_edge_stay_in_the_order_the_platform_found_them_in() {
    // Arrange -- two monitors stacked on the same left edge, with a third
    // to their right, found once bottom first and once top first.
    let top = monitor(0, 0, 0);
    let bottom = monitor(1, 0, 1080);
    let right = monitor(2, 1920, 0);
    let bottom_first = FakeEnumerator::finding(vec![right.clone(), bottom.clone(), top.clone()]);
    let top_first = FakeEnumerator::finding(vec![top, right, bottom]);

    // Act
    let from_bottom = bottom_first.monitors().expect("the monitors can be listed");
    let from_top = top_first.monitors().expect("the monitors can be listed");

    // Assert
    assert_eq!(ids(&from_bottom), vec![1, 0, 2]);
    assert_eq!(ids(&from_top), vec![0, 1, 2]);
}

#[test]
fn a_monitor_list_that_cannot_be_read_is_an_error() {
    // Arrange
    let platform = FakeEnumerator { found: None };

    // Act
    let result = platform.monitors();

    // Assert
    assert!(matches!(result, Err(Error::Unsupported(_))));
}
