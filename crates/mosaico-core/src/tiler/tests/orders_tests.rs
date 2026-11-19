//! Orders naming the window, monitor or workspace they act on.

use crate::MonitorId;
use crate::id::wid;
use crate::layout::LayoutKind;
use crate::tiler::{Order, TilingManager};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, still};

/// Windows 1 and 2 tiled on monitor 0, 3 on monitor 1, and 1 focused.
fn two_monitors(platform: &FakePlatform) -> TilingManager<FakePlatform> {
    start(platform, None, &still())
}

fn monitor(manager: &TilingManager<FakePlatform>, index: usize) -> MonitorId {
    manager.tiler().monitors[index].id
}

#[test]
fn an_order_for_a_window_that_is_not_managed_is_refused() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);

    // Act
    let done = manager.order(Order::Focus { window: wid(99) });

    // Assert
    assert!(!done);
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
}

#[test]
fn focus_brings_a_window_on_another_monitor_into_focus() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);

    // Act
    let done = manager.order(Order::Focus { window: wid(3) });

    // Assert
    assert!(done);
    assert_eq!(manager.tiler().focused_window(), Some(wid(3)));
    assert_eq!(manager.tiler().focused_monitor(), 1);
}

#[test]
fn send_moves_a_window_that_is_not_focused_and_follows_it() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);

    // Act
    let done = manager.order(Order::Send {
        window: wid(1),
        workspace: 2,
    });

    // Assert
    assert!(done);
    let mon = &manager.tiler().monitors[0];
    assert!(mon.workspaces[2].contains(wid(1)));
    assert!(!mon.workspaces[0].contains(wid(1)));
    assert_eq!(mon.active_workspace, 2);
}

#[test]
fn float_floats_a_window_and_tiles_it_again() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);

    // Act
    let floated = manager.order(Order::Float {
        window: wid(1),
        floating: true,
    });
    let floats = manager.tiler().monitors[0].workspaces[0].is_floating(wid(1));
    let again = manager.order(Order::Float {
        window: wid(1),
        floating: true,
    });
    let still_floats = manager.tiler().monitors[0].workspaces[0].is_floating(wid(1));
    manager.order(Order::Float {
        window: wid(1),
        floating: false,
    });

    // Assert -- asking twice does not toggle it back.
    assert!(floated && again);
    assert!(floats && still_floats);
    assert!(manager.tiler().monitors[0].workspaces[0].contains(wid(1)));
}

#[test]
fn close_asks_the_window_to_close_without_focusing_it() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);
    platform.take_calls();

    // Act
    let done = manager.order(Order::Close { window: wid(2) });

    // Assert
    assert!(done);
    assert!(platform.calls().contains(&Call::Close(wid(2))));
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
}

#[test]
fn show_switches_the_workspace_of_the_monitor_named() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);
    let second = monitor(&manager, 1);

    // Act
    let done = manager.order(Order::Show {
        monitor: second,
        workspace: 3,
    });

    // Assert -- focus stays on the monitor the user is on.
    assert!(done);
    assert_eq!(manager.tiler().monitors[1].active_workspace, 3);
    assert_eq!(manager.tiler().monitors[0].active_workspace, 0);
    assert_eq!(manager.tiler().focused_monitor(), 0);
}

#[test]
fn show_refuses_a_workspace_the_monitor_does_not_have() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);
    let first = monitor(&manager, 0);

    // Act
    let done = manager.order(Order::Show {
        monitor: first,
        workspace: 99,
    });

    // Assert
    assert!(!done);
}

#[test]
fn layout_changes_a_workspace_out_of_view_without_showing_it() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);
    let first = monitor(&manager, 0);

    // Act
    let done = manager.order(Order::Layout {
        monitor: first,
        workspace: 4,
        layout: LayoutKind::ThreeColumn,
    });

    // Assert
    assert!(done);
    let mon = &manager.tiler().monitors[0];
    assert_eq!(mon.workspaces[4].layout_kind(), LayoutKind::ThreeColumn);
    assert_eq!(mon.active_workspace, 0);
}

#[test]
fn monocle_turns_on_for_the_monitor_named_and_only_once() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = two_monitors(&platform);
    let second = monitor(&manager, 1);

    // Act
    manager.order(Order::Monocle {
        monitor: second,
        on: true,
    });
    manager.order(Order::Monocle {
        monitor: second,
        on: true,
    });

    // Assert
    let mon = &manager.tiler().monitors[1];
    assert!(mon.active_ws().monocle());
    assert_eq!(mon.active_ws().monocle_window(), Some(wid(3)));
    assert!(!manager.tiler().monitors[0].active_ws().monocle());
}

#[test]
fn orders_travel_as_json_named_by_their_order_field() {
    // Arrange
    let order = Order::Send {
        window: wid(0x4014E),
        workspace: 2,
    };

    // Act
    let json = serde_json::to_value(&order).unwrap();

    // Assert
    assert_eq!(json["order"], "send");
    assert_eq!(serde_json::from_value::<Order>(json).unwrap(), order);
}
