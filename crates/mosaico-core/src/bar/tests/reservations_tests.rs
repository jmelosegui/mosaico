//! The space bars reserve through the bar API, kept by session.

use std::time::{Duration, Instant};

use crate::api::{MonitorTarget, Peer};
use crate::tiler::{Edges, MonitorPlace};
use crate::{Rect, StableMonitorId};

use super::Reservations;

fn id(text: &str) -> StableMonitorId {
    StableMonitorId::new(text)
}

fn places() -> Vec<MonitorPlace> {
    let place = |index: usize, text: &str| {
        let bounds = Rect::new(1920 * index as i32, 0, 1920, 1080);
        MonitorPlace {
            id: id(text),
            index,
            bounds,
            work_area: bounds,
            scale: 1.0,
        }
    };
    vec![place(0, "m-a"), place(1, "m-b")]
}

fn peer(session: u64, process: u32) -> Peer {
    Peer {
        session,
        bar: Some("yasb".to_string()),
        process: Some(process),
    }
}

fn top(pixels: i32) -> Edges {
    Edges::new(pixels, 0, 0, 0)
}

#[test]
fn reservations_on_one_edge_keep_the_furthest_clear() {
    // Arrange
    let mut space = Reservations::default();

    // Act
    space.set(&peer(1, 10), MonitorTarget::All, top(32));
    space.set(&peer(2, 20), MonitorTarget::One(id("m-b")), top(48));

    // Assert
    let reach = space.reach(&places());
    assert_eq!((reach.top(&id("m-a")), reach.top(&id("m-b"))), (32, 48));
}

#[test]
fn a_session_reserving_a_monitor_again_replaces_what_it_had_there() {
    // Arrange
    let mut space = Reservations::default();
    space.set(&peer(1, 10), MonitorTarget::One(id("m-a")), top(48));

    // Act
    space.set(&peer(1, 10), MonitorTarget::One(id("m-a")), top(24));

    // Assert
    assert_eq!(space.reach(&places()).top(&id("m-a")), 24);
}

#[test]
fn a_reservation_of_no_edges_ends_it() {
    // Arrange
    let mut space = Reservations::default();
    space.set(&peer(1, 10), MonitorTarget::All, top(48));

    // Act
    space.set(&peer(1, 10), MonitorTarget::All, Edges::default());

    // Assert
    assert!(space.reach(&places()).is_none());
    assert_eq!(space.processes(), Vec::<u32>::new());
}

#[test]
fn a_release_ends_only_what_its_session_holds_on_the_monitor_named() {
    // Arrange
    let mut space = Reservations::default();
    space.set(&peer(1, 10), MonitorTarget::One(id("m-a")), top(40));
    space.set(&peer(1, 10), MonitorTarget::One(id("m-b")), top(40));
    space.set(&peer(2, 20), MonitorTarget::One(id("m-a")), top(30));

    // Act
    let released = space.release(1, Some(&MonitorTarget::One(id("m-a"))));

    // Assert
    let reach = space.reach(&places());
    assert!(released);
    assert_eq!((reach.top(&id("m-a")), reach.top(&id("m-b"))), (30, 40));
}

#[test]
fn the_space_of_an_ended_session_is_kept_until_its_grace_is_over() {
    // Arrange
    let mut space = Reservations::default();
    space.set(&peer(1, 10), MonitorTarget::All, top(40));
    let now = Instant::now();
    let until = now + Duration::from_secs(5);

    // Act
    space.end(1, until);
    let early = space.expire(now);

    // Assert
    assert!(!early);
    assert_eq!(space.due(), Some(until));
    assert!(space.expire(until));
    assert!(space.reach(&places()).is_none());
    assert_eq!(space.due(), None);
}

#[test]
fn an_edge_past_a_quarter_of_the_monitor_is_left_out() {
    // Arrange -- refused when asked for, but a monitor may have shrunk
    // since.
    let mut space = Reservations::default();
    space.set(&peer(1, 10), MonitorTarget::All, Edges::new(400, 30, 0, 0));

    // Act
    let reach = space.reach(&places());

    // Assert
    assert_eq!(reach.edges(&id("m-a")), Edges::new(0, 30, 0, 0));
}

#[test]
fn the_bar_processes_are_listed_once_and_follow_what_is_kept() {
    // Arrange
    let mut space = Reservations::default();
    space.set(&peer(1, 20), MonitorTarget::One(id("m-a")), top(40));
    space.set(&peer(1, 20), MonitorTarget::One(id("m-b")), top(40));
    let mut other = peer(2, 10);
    other.bar = Some("zebar".to_string());
    space.set(&other, MonitorTarget::All, top(30));

    // Act
    let before = space.processes();
    let dropped = space.retain(|bar| bar == Some("yasb"));

    // Assert
    assert_eq!(before, vec![10, 20]);
    assert!(dropped);
    assert_eq!(space.processes(), vec![20]);
}
