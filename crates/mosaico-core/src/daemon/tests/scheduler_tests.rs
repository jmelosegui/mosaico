//! Scheduler tests against a fake clock: every `now` is `t0` plus an
//! offset, and tasks are named after what they stand for.

use std::time::{Duration, Instant};

use crate::WindowEvent;
use crate::daemon::SETTLE;
use crate::id::wid;

use super::*;

#[path = "scheduler_flush_tests.rs"]
mod flush_tests;
#[path = "scheduler_settle_tests.rs"]
mod settle_tests;

type Task = &'static str;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn received(messages: Vec<Message<Task>>) -> Wake<Task> {
    Wake::Received(messages)
}

fn timed_out() -> Wake<Task> {
    Wake::TimedOut
}

fn task(name: Task) -> Message<Task> {
    Message::Task(name)
}

fn event(event: WindowEvent) -> Message<Task> {
    Message::Event(event)
}

fn focused(window: u64) -> WindowEvent {
    WindowEvent::Focused {
        window: wid(window),
    }
}

fn moved(window: u64) -> WindowEvent {
    WindowEvent::LocationChanged {
        window: wid(window),
    }
}

fn display_changed() -> Wake<Task> {
    received(vec![event(WindowEvent::DisplayChanged)])
}

#[test]
fn tasks_run_before_the_window_events_that_arrived_with_them() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let created = WindowEvent::Created { window: wid(2) };
    let batch = received(vec![
        event(focused(1)),
        task("hotkey"),
        event(created.clone()),
        task("command"),
    ]);

    // Act
    let steps = scheduler.step(batch, t0);

    // Assert
    assert_eq!(
        steps,
        vec![
            Step::Run("hotkey"),
            Step::Run("command"),
            Step::Events(vec![focused(1), created]),
        ]
    );
}

#[test]
fn a_batch_loses_the_repeats_that_say_nothing_new() {
    // Arrange -- a window reporting its position on every frame.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let batch = received(vec![
        event(moved(1)),
        event(focused(2)),
        event(moved(1)),
        event(moved(1)),
    ]);

    // Act
    let steps = scheduler.step(batch, t0);

    // Assert
    assert_eq!(steps, vec![Step::Events(vec![focused(2), moved(1)])]);
}

#[test]
fn a_batch_without_window_events_only_runs_its_tasks() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();

    // Act
    let steps = scheduler.step(received(vec![task("tick")]), t0);

    // Assert
    assert_eq!(steps, vec![Step::Run("tick")]);
}

#[test]
fn an_idle_loop_waits_for_messages_and_does_nothing_when_it_times_out() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();

    // Act
    let wait = scheduler.begin_wait(None, t0);
    let steps = scheduler.step(timed_out(), t0 + ms(50));

    // Assert
    assert_eq!(wait, None);
    assert!(steps.is_empty(), "{steps:?}");
}
