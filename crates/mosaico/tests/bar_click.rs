//! Clicking and scrolling the status bar on the live desktop.
//!
//! Like `drag_and_drop.rs`, these tests start a daemon of their own, which
//! tiles the live desktop, and they move the real pointer, press its left
//! button and turn its wheel, so they run on demand, on the desktop
//! runner, never with the safe suites. The bar on the primary monitor is
//! drawn as the tests' own `bar.toml` has it, and what the pointer clicks
//! on it is found the way the user finds it: left to right, where the
//! pointer shows the hand. The pointer only presses or turns the wheel
//! where it shows the hand over that bar.

#[path = "bar/win32.rs"]
mod bar_win32;
mod common;
#[path = "bar/hand.rs"]
mod hand;
#[path = "drag/pointer.rs"]
mod pointer;
#[path = "bar/strip.rs"]
mod strip;
// The bar tests make only some of the calls the drag tests make.
#[allow(dead_code)]
#[path = "drag/win32.rs"]
mod win32;
#[path = "bar/workspace.rs"]
mod workspace;

use mosaico_core::pid::PidFile;
use mosaico_core::system::ProcessSetup;
use mosaico_windows::setup::WindowsSetup;

use common::{RunningDaemon, TestInstance, desktop_guard};
use hand::Run;
use strip::Bar;
use workspace::shown;

/// How many workspaces a monitor has, and so how many labels its bar has.
const LABELS: usize = 8;

/// What the pointer clicks on the bar: the workspace labels, in order,
/// then the layout widget.
struct Items {
    labels: Vec<Run>,
    layout: Run,
}

/// Starts a daemon with the tests' bar, transitions off and the pointer
/// left where the tests put it, finds the items of its bar on the primary
/// monitor, which shows workspace 1, and runs `case` with them, before the
/// daemon is stopped and the pointer goes back where the case found it.
fn on_the_bar(label: &str, case: impl FnOnce(&TestInstance, &Bar, &Items)) {
    let instance = TestInstance::new(label);
    let config = "[animations]\nenabled = false\n\n[mouse]\nfollows_focus = false\n";
    instance.write_config("config.toml", config);
    instance.write_config("bar.toml", include_str!("bar/bar.toml"));
    // Read and move the pointer in the physical pixels the daemon works in.
    WindowsSetup.init();
    let _pointer = pointer::Resting::here();
    instance.start_daemon();
    let _daemon = RunningDaemon::new(&instance);
    let pid_path = instance.root().join("data").join("mosaico.pid");
    let pid = PidFile::new(pid_path).read().ok().flatten();
    let bar = strip::primary(pid.expect("the daemon wrote no PID file"));
    assert_eq!(
        shown(&instance, &bar, 1),
        Some(1),
        "the monitor of the bar at {bar:?} is not on workspace 1"
    );

    let mut labels = hand::runs(&bar, bar.left, LABELS + 1);
    assert_eq!(labels.len(), LABELS + 1, "the hand showed over {labels:?}");
    let layout = labels.pop().expect("the runs were counted");
    case(&instance, &bar, &Items { labels, layout });
}

/// Clicks the middle of `run` with the left button, once the pointer
/// shows the hand there.
fn click(bar: &Bar, run: Run) {
    pointer::click(bar.at(run.middle()), hand::takes(bar));
}

#[test]
fn a_click_on_a_workspace_label_goes_to_that_workspace() {
    // Arrange
    let _guard = desktop_guard();
    on_the_bar("bar-click", |instance, bar, items| {
        // Act -- on label 2, then on label 1.
        click(bar, items.labels[1]);
        let there = shown(instance, bar, 2);
        click(bar, items.labels[0]);
        let back = shown(instance, bar, 1);

        // Assert
        assert_eq!((there, back), (Some(2), Some(1)));
    });
}

#[test]
fn a_notch_of_the_wheel_steps_to_the_next_workspace_and_back() {
    // Arrange -- the wheel goes to the bar only under the pointer.
    let _guard = desktop_guard();
    let off = "turn on Scroll inactive windows when I hover over them";
    assert!(hand::wheel_goes_under_the_pointer(), "{off}");
    on_the_bar("bar-wheel", |instance, bar, items| {
        // Act -- a notch down over label 1, then a notch up.
        let over = bar.at(items.labels[0].middle());
        pointer::wheel(over, 1, hand::takes(bar));
        let next = shown(instance, bar, 2);
        pointer::wheel(over, -1, hand::takes(bar));
        let back = shown(instance, bar, 1);

        // Assert
        assert_eq!((next, back), (Some(2), Some(1)));
    });
}

#[test]
fn a_click_on_the_layout_widget_cycles_the_layout() {
    // Arrange -- on workspace 8, empty, so a new layout moves no window.
    let _guard = desktop_guard();
    on_the_bar("bar-layout", |instance, bar, items| {
        click(bar, items.labels[7]);
        assert_eq!(shown(instance, bar, 8), Some(8), "not on workspace 8");
        let left = items.layout.left;
        let mut widths = vec![hand::width_at(bar, left)];

        // Act -- a click for each layout, reading the widget after each.
        for _ in 0..3 {
            click(bar, items.layout);
            let before = widths[widths.len() - 1];
            widths.push(hand::width_after(bar, left, before));
        }
        click(bar, items.labels[0]);

        // Assert -- the widget names a layout of another width, and three
        // clicks later the one it named first.
        assert_ne!(widths[1], widths[0], "a click changed nothing: {widths:?}");
        assert_eq!(widths[3], widths[0], "not back to the first: {widths:?}");
        assert_eq!(shown(instance, bar, 1), Some(1), "not back on workspace 1");
    });
}
