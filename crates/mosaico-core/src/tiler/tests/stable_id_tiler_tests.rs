//! The ids monitors keep through display changes, and the reach that
//! follows them.

use std::time::Instant;

use crate::{MonitorId, MonitorKey, Rect};

use super::super::{Display, Input, MonitorPlace, Reach, Settings, Tiler};
use super::FakeDesktop;

fn key(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

/// Monitors with `keys`, left to right, with session ids from `id_base`
/// as a platform may hand out new ones on every change.
fn shown(keys: &[&MonitorKey], id_base: u64) -> Vec<Display> {
    (keys.iter().enumerate())
        .map(|(i, k)| {
            let area = Rect::new(i as i32 * 1920, 0, 1920, 1080);
            Display {
                id: MonitorId::from_raw(id_base + i as u64),
                key: (*k).clone(),
                bounds: area,
                work_area: area,
                primary: i == 0,
                name: String::new(),
                device: String::new(),
                scale: 1.0,
            }
        })
        .collect()
}

fn started(displays: &[Display]) -> (Tiler, FakeDesktop) {
    let desktop = FakeDesktop::default();
    let (tiler, _) = Tiler::new(Settings::default(), displays, &[], &desktop, Instant::now());
    (tiler, desktop)
}

fn change(tiler: &mut Tiler, desktop: &FakeDesktop, monitors: Vec<Display>) {
    let input = Input::Displays {
        monitors,
        reach: Reach::none(),
    };
    tiler.step(input, desktop, Instant::now());
}

fn ids(places: &[MonitorPlace]) -> Vec<String> {
    places.iter().map(|p| p.id.to_string()).collect()
}

#[test]
fn a_monitor_keeps_its_id_when_its_session_id_changes() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let (mut tiler, desktop) = started(&shown(&[&a, &b], 1));
    let before = ids(&tiler.places());

    // Act
    change(&mut tiler, &desktop, shown(&[&a, &b], 10));

    // Assert
    assert_eq!(ids(&tiler.places()), before);
}

#[test]
fn a_monitor_that_comes_back_gets_its_id_back() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let (mut tiler, desktop) = started(&shown(&[&a, &b], 1));
    let before = ids(&tiler.places());

    // Act
    change(&mut tiler, &desktop, shown(&[&a], 10));
    change(&mut tiler, &desktop, shown(&[&a, &b], 20));

    // Assert
    assert_eq!(ids(&tiler.places()), before);
}

#[test]
fn two_monitors_that_look_the_same_get_different_ids() {
    // Arrange
    let same = MonitorKey::default();

    // Act
    let (tiler, _) = started(&shown(&[&same, &same], 1));

    // Assert
    let ids = ids(&tiler.places());
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn the_places_for_a_change_are_the_places_after_it() {
    // Arrange -- B leaves, then comes back to the left of a new C.
    let (a, b, c) = (key("A"), key("B"), key("C"));
    let (mut tiler, desktop) = started(&shown(&[&a, &b], 1));
    change(&mut tiler, &desktop, shown(&[&a], 10));
    let next = shown(&[&b, &a, &c], 20);

    // Act
    let preview = tiler.places_for(&next);
    change(&mut tiler, &desktop, next);

    // Assert
    assert_eq!(preview, tiler.places());
}

#[test]
fn a_reach_follows_its_monitor_when_the_monitors_swap() {
    // Arrange -- the bar is only on B, which then moves to the left.
    let (a, b) = (key("A"), key("B"));
    let (mut tiler, desktop) = started(&shown(&[&a, &b], 1));
    let next = shown(&[&b, &a], 10);
    let reach = Reach::bar_on(&tiler.places_for(&next), 30, &[0]);

    // Act
    let input = Input::Displays {
        monitors: next,
        reach,
    };
    tiler.step(input, &desktop, Instant::now());

    // Assert
    let tops: Vec<i32> = tiler.monitors().iter().map(|m| m.work_area().y).collect();
    assert_eq!(tops, [30, 0]);
    assert_eq!(tiler.monitors()[0].key(), &b);
}
