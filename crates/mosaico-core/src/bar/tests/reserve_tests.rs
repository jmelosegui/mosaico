use std::collections::BTreeMap;

use super::*;
use crate::{Rect, StableMonitorId};

fn place(index: usize, id: &str, scale: f64) -> MonitorPlace {
    MonitorPlace {
        id: StableMonitorId::new(id),
        index,
        bounds: Rect::new(index as i32 * 1920, 0, 1920, 1080),
        work_area: Rect::new(index as i32 * 1920, 0, 1920, 1040),
        scale,
    }
}

fn edges(top: i32, bottom: i32, left: i32, right: i32) -> ReserveEdges {
    ReserveEdges {
        top,
        bottom,
        left,
        right,
    }
}

fn reserve(top: i32, monitor: &[(&str, ReserveEdges)]) -> BarReserve {
    BarReserve {
        top,
        monitor: (monitor.iter())
            .map(|(name, edges)| (name.to_string(), *edges))
            .collect::<BTreeMap<_, _>>(),
        ..BarReserve::default()
    }
}

#[test]
fn every_monitor_keeps_the_global_edges_times_its_scale() {
    // Arrange
    let reserve = reserve(30, &[]);
    let places = [place(0, "m-a", 1.0), place(1, "m-b", 1.5)];

    // Act
    let reach = reserve_reach(&reserve, &places);

    // Assert
    assert_eq!(reach.top(&StableMonitorId::new("m-a")), 30);
    assert_eq!(reach.top(&StableMonitorId::new("m-b")), 45);
}

#[test]
fn a_monitor_table_replaces_the_global_edges_on_that_monitor_only() {
    // Arrange
    let reserve = reserve(30, &[("1", edges(0, 40, 0, 0))]);
    let places = [place(0, "m-a", 1.0), place(1, "m-b", 1.0)];

    // Act
    let reach = reserve_reach(&reserve, &places);

    // Assert
    assert_eq!(
        reach.edges(&StableMonitorId::new("m-a")),
        Edges::new(30, 0, 0, 0)
    );
    assert_eq!(
        reach.edges(&StableMonitorId::new("m-b")),
        Edges::new(0, 40, 0, 0)
    );
}

#[test]
fn a_monitor_named_by_id_wins_over_one_named_by_position() {
    // Arrange
    let reserve = reserve(0, &[("0", edges(10, 0, 0, 0)), ("m-a", edges(20, 0, 0, 0))]);

    // Act
    let space = ReservedSpace::on(&reserve, &place(0, "m-a", 1.0));

    // Assert
    assert_eq!(space.edges, Edges::new(20, 0, 0, 0));
}

#[test]
fn an_edge_past_a_quarter_of_the_monitor_is_refused_and_keeps_nothing() {
    // Arrange
    let reserve = BarReserve {
        top: 30,
        left: 600,
        ..BarReserve::default()
    };

    // Act
    let space = ReservedSpace::on(&reserve, &place(0, "m-a", 1.0));

    // Assert
    assert_eq!(space.refused, vec![Side::Left]);
    assert_eq!(space.edges, Edges::new(30, 0, 0, 0));
}

#[test]
fn a_reserve_of_nothing_reaches_nowhere() {
    // Arrange
    let places = [place(0, "m-a", 1.0)];

    // Act
    let reach = reserve_reach(&BarReserve::default(), &places);

    // Assert
    assert_eq!(reach, Reach::none());
}

#[test]
fn monitor_tables_naming_no_connected_monitor_are_listed() {
    // Arrange -- one table by id, one by position, one by neither.
    let reserve = reserve(
        0,
        &[
            ("m-b", edges(1, 0, 0, 0)),
            ("0", edges(1, 0, 0, 0)),
            ("gone", edges(1, 0, 0, 0)),
            ("5", edges(1, 0, 0, 0)),
        ],
    );
    let places = [place(0, "m-a", 1.0), place(1, "m-b", 1.0)];

    // Act
    let unmatched = unmatched_monitors(&reserve, &places);

    // Assert
    assert_eq!(unmatched, ["5", "gone"]);
}
