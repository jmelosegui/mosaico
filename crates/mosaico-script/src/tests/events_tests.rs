//! Handlers, waiting, timers and stopping.

use std::time::Duration;

use mosaico_core::desktop_events::DesktopEvent;

use super::support::{World, mid, wid};
use crate::{Options, Outcome};

fn switched(workspace: usize) -> DesktopEvent {
    DesktopEvent::WorkspaceChanged {
        monitor: mid(10),
        workspace,
    }
}

fn focused(window: u64) -> DesktopEvent {
    DesktopEvent::FocusChanged {
        window: Some(wid(window)),
        monitor: mid(10),
    }
}

#[test]
fn a_script_without_handlers_or_timers_finishes() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run("print('done')");

    // Assert
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(world.daemon.subscribed.get(), 0);
}

#[test]
fn handlers_get_their_events_with_workspaces_from_one() {
    // Arrange
    let world = World::new();
    world.events(vec![switched(2), focused(1), switched(0)]);

    // Act
    let outcome = world.run(
        r#"
        mosaico.on("workspace_changed", function(e)
            print(e.event, e.monitor, e.workspace)
        end)
        "#,
    );

    // Assert -- the daemon went away after the last event.
    assert_eq!(outcome, Outcome::DaemonGone);
    assert_eq!(
        world.out(),
        "workspace_changed\t10\t3\nworkspace_changed\t10\t1\n"
    );
}

#[test]
fn an_unknown_event_name_fails_the_script() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run(r#"mosaico.on("focus_change", function() end)"#);

    // Assert
    let Outcome::Failed(message) = outcome else {
        panic!("expected a failure, got {outcome:?}");
    };
    assert!(
        message.contains("unknown event `focus_change`"),
        "{message}"
    );
}

#[test]
fn after_the_daemon_comes_back_the_handlers_go_on() {
    // Arrange
    let world = World::new();
    world.events(vec![switched(1)]);
    world.events(vec![switched(2)]);
    let mut script =
        world.script(r#"mosaico.on("workspace_changed", function(e) print(e.workspace) end)"#);

    // Act
    let first = script.run();
    let second = script.run();

    // Assert
    assert_eq!((first, second), (Outcome::DaemonGone, Outcome::DaemonGone));
    assert_eq!(world.out(), "2\n3\n");
    assert_eq!(world.daemon.subscribed.get(), 2);
}

#[test]
fn wait_returns_the_event_asked_for_and_keeps_the_others() {
    // Arrange
    let world = World::new();
    world.events(vec![focused(1), switched(2), focused(2)]);

    // Act
    world.run(
        r#"
        mosaico.on("focus_changed", function(e) print("handler", e.window) end)
        local e = mosaico.wait("focus_changed", { window = 2 }, 5)
        print("waited", e.window)
        "#,
    );

    // Assert -- the focus on 1 waited for the script's file to finish.
    assert_eq!(world.out(), "waited\t2\nhandler\t1\n");
}

#[test]
fn wait_gives_up_at_its_timeout() {
    // Arrange
    let world = World::new();
    world.events(vec![]);

    // Act
    world.run(r#"print(mosaico.wait("monitor_added", 3))"#);

    // Assert
    assert_eq!(world.out(), "nil\ttimed out\n");
    assert_eq!(world.elapsed(), Duration::from_secs(3));
}

#[test]
fn timers_fire_in_order_and_every_repeats_until_cancelled() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run(
        r#"
        local n = 0
        local tick
        tick = mosaico.every(1, function()
            n = n + 1
            print("tick", n)
            if n == 3 then mosaico.cancel(tick) end
        end)
        mosaico.after(1.5, function() print("once") end)
        "#,
    );

    // Assert
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(world.out(), "tick\t1\nonce\ntick\t2\ntick\t3\n");
    assert_eq!(world.elapsed(), Duration::from_secs(3));
}

#[test]
fn stop_ends_the_script_with_its_code() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run(
        r#"
        mosaico.after(2, function() mosaico.stop(3) end)
        mosaico.every(10, function() print("never") end)
        "#,
    );

    // Assert
    assert_eq!(outcome, Outcome::Stopped(3));
    assert_eq!(world.out(), "");
}

#[test]
fn an_error_in_a_handler_is_reported_and_the_script_goes_on() {
    // Arrange
    let world = World::new();
    world.events(vec![switched(1), switched(2)]);

    // Act
    let outcome = world.run(
        r#"
        mosaico.on("workspace_changed", function(e)
            if e.workspace == 2 then error("boom") end
            print(e.workspace)
        end)
        "#,
    );

    // Assert
    assert_eq!(outcome, Outcome::DaemonGone);
    assert_eq!(world.out(), "3\n");
    assert!(world.err().contains("boom"), "{}", world.err());
}

#[test]
fn fail_fast_stops_at_the_first_error_in_a_handler() {
    // Arrange
    let world = World::new();
    world.events(vec![switched(1), switched(2)]);
    let options = Options {
        fail_fast: true,
        ..Options::default()
    };

    // Act
    let outcome = world
        .script_with(
            r#"mosaico.on("workspace_changed", function() error("boom") end)"#,
            options,
        )
        .run();

    // Assert
    assert!(matches!(outcome, Outcome::Failed(m) if m.contains("boom")));
}

#[test]
fn sleep_lets_the_clock_run_and_keeps_events_for_later() {
    // Arrange
    let world = World::new();
    world.events(vec![switched(1)]);

    // Act
    world.run(
        r#"
        mosaico.on("workspace_changed", function(e) print("switched", e.workspace) end)
        mosaico.sleep(2)
        print("slept")
        "#,
    );

    // Assert
    assert_eq!(world.out(), "slept\nswitched\t2\n");
    assert_eq!(world.elapsed(), Duration::from_secs(2));
}
