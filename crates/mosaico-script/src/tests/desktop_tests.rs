//! Reading the desktop and acting on it.

use mosaico_core::Action;
use mosaico_core::action::Direction;
use mosaico_core::ipc::{Command, ErrorCode, Response};
use mosaico_core::layout::LayoutKind;
use mosaico_core::tiler::Order;

use super::support::{World, mid, wid};
use crate::Outcome;

#[test]
fn state_counts_workspaces_from_one() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run(
        r#"
        local s = mosaico.state()
        print(s.focused_window, s.focused_monitor, s.monitors[1].workspace, #s.windows)
        local t = s.windows[3]
        print(t.class, t.workspace, t.floating, s.monitors[1].workspaces[1].layout)
        "#,
    );

    // Assert
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(world.out(), "2\t10\t2\t3\nTerminal\t2\ttrue\tbsp\n");
}

#[test]
fn windows_are_filtered_by_class_title_and_workspace() {
    // Arrange
    let world = World::new();

    // Act
    world.run(
        r#"
        print(#mosaico.windows(), #mosaico.windows{ class = "Code" })
        print(mosaico.windows{ title = "Note" }[1].id, #mosaico.windows{ workspace = 2 })
        print(#mosaico.windows{ floating = true, monitor = 10 })
        "#,
    );

    // Assert
    assert_eq!(world.out(), "3\t1\n1\t1\n1\n");
}

#[test]
fn an_action_is_sent_by_its_name() {
    // Arrange
    let world = World::new();

    // Act
    world.run(r#"print(mosaico.action("focus-left"))"#);

    // Assert
    assert_eq!(world.out(), "true\n");
    assert_eq!(
        world.sent(),
        vec![Command::Action {
            action: Action::Focus(Direction::Left)
        }]
    );
}

#[test]
fn an_unknown_action_returns_nil_and_why() {
    // Arrange
    let world = World::new();

    // Act
    world.run(r#"print(mosaico.action("fly"))"#);

    // Assert
    assert_eq!(world.out(), "nil\tunknown action `fly`\n");
    assert!(world.sent().is_empty());
}

#[test]
fn orders_name_their_window_monitor_and_workspace() {
    // Arrange
    let world = World::new();

    // Act
    world.run(
        r#"
        mosaico.focus(3)
        mosaico.send(1, 3)
        mosaico.float(2)
        mosaico.float(2, false)
        mosaico.close(1)
        mosaico.show(20, 2)
        mosaico.layout(10, 1, "three-column")
        mosaico.monocle(10, false)
        "#,
    );

    // Assert -- workspaces go to the daemon counted from 0.
    let orders: Vec<Order> = world
        .sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::Order { order } => Some(order),
            _ => None,
        })
        .collect();
    assert_eq!(
        orders,
        vec![
            Order::Focus { window: wid(3) },
            Order::Send {
                window: wid(1),
                workspace: 2
            },
            Order::Float {
                window: wid(2),
                floating: true
            },
            Order::Float {
                window: wid(2),
                floating: false
            },
            Order::Close { window: wid(1) },
            Order::Show {
                monitor: mid(20),
                workspace: 1
            },
            Order::Layout {
                monitor: mid(10),
                workspace: 0,
                layout: LayoutKind::ThreeColumn
            },
            Order::Monocle {
                monitor: mid(10),
                on: false
            },
        ]
    );
}

#[test]
fn an_order_that_makes_no_sense_is_not_sent() {
    // Arrange
    let world = World::new();

    // Act
    world.run(
        r#"
        print(mosaico.send(1, 0))
        print(mosaico.layout(10, 1, "spiral"))
        print(mosaico.focus("one"))
        "#,
    );

    // Assert
    assert_eq!(
        world.out(),
        "nil\texpected a workspace number, counted from 1\n\
         nil\tunknown layout `spiral`; use bsp, vertical-stack or three-column\n\
         nil\texpected a window id\n"
    );
    assert!(world.sent().is_empty());
}

#[test]
fn a_refusal_or_a_missing_daemon_returns_nil_and_why() {
    // Arrange
    let world = World::new();
    world
        .daemon
        .replies
        .borrow_mut()
        .push_back(Response::error(ErrorCode::NotFound, "no such window"));

    // Act
    world.run(r#"print(mosaico.focus(99))"#);
    world.daemon.running.set(false);
    world.run(r#"print(mosaico.state())"#);

    // Assert
    assert_eq!(
        world.out(),
        "nil\tno such window\nnil\tmosaico is not running\n"
    );
}
