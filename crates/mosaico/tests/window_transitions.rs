//! Window transitions on the live desktop.
//!
//! Like `window_management.rs`, these tests start a daemon of their own,
//! launch notepad and tile the live desktop, so they run on demand, on the
//! desktop runner, never with the safe suites. Two notepads side by side
//! on an empty workspace are swapped with `mosaico action move`, and where
//! both windows are is sampled every few milliseconds while they swap.

mod common;
#[path = "transitions/notepads.rs"]
mod notepads;
#[path = "transitions/premise.rs"]
mod premise;

use std::process::Stdio;
use std::thread;
use std::time::Duration;

use mosaico_core::system::ProcessSetup;
use mosaico_core::{Rect, WindowId};
use mosaico_windows::setup::WindowsSetup;

use common::{RunningDaemon, TestInstance, desktop_guard};
use notepads::Notepads;

/// How long a transition takes in the test, long enough for many frames.
const DURATION_MS: u64 = 400;

/// How long a swap is watched, from just before it is asked for.
const WATCHED: Duration = Duration::from_millis(1200);

/// How many places between its slots each window must be seen at.
const MIN_BETWEEN: usize = 4;

/// One swap of two windows: where each was before and after, and every
/// rect it was seen at while they swapped.
struct Swap {
    before: [Rect; 2],
    after: [Rect; 2],
    seen: [Vec<Rect>; 2],
}

/// Opens two notepads on workspace 8 with `config.toml` holding `config`,
/// and swaps them with the move action of the one the tiler has focused.
fn swap_with(label: &str, config: &str) -> Swap {
    let instance = TestInstance::new(label);
    instance.write_config("config.toml", config);
    // Read windows in the physical pixels the daemon works in.
    WindowsSetup.init();
    instance.start_daemon();
    let _daemon = RunningDaemon::new(&instance);
    instance.run(&["action", "go-to-workspace", "8"]);
    thread::sleep(Duration::from_millis(500));

    let mut notepads = Notepads::new();
    let windows = [notepads.open(), notepads.open()];
    let before = notepads.still(windows);
    let state = instance.stdout(&["debug", "state"]);
    premise::assert_alone_side_by_side(&state, windows, before);
    let focus = |w: &WindowId| state.lines().any(|l| l == format!("Focused window: {w}"));
    let focused = windows
        .iter()
        .position(focus)
        .unwrap_or_else(|| panic!("neither notepad has the tiler's focus:\n{state}"));
    let toward = if before[focused].x < before[1 - focused].x {
        "right"
    } else {
        "left"
    };

    let mut action = instance.command(&["action", "move", toward]);
    let child = action.stdin(Stdio::null()).stdout(Stdio::null()).spawn();
    let mut child = child.expect("failed to run mosaico action move");
    let seen = notepads.sample(windows, WATCHED);
    assert!(child.wait().is_ok_and(|s| s.success()), "the move failed");
    let after = notepads.still(windows);
    Swap {
        before,
        after,
        seen,
    }
}

/// Whether every edge of `rect` lies between those of `from` and `to`.
fn within(rect: Rect, from: Rect, to: Rect) -> bool {
    let edges = |r: Rect| [r.x, r.y, r.x + r.width, r.y + r.height];
    let (rect, from, to) = (edges(rect), edges(from), edges(to));
    (0..4).all(|i| from[i].min(to[i]) <= rect[i] && rect[i] <= from[i].max(to[i]))
}

#[test]
fn a_swap_moves_both_windows_through_the_box_between_their_slots() {
    // Arrange -- linear, and whatever the system's animation setting.
    let _guard = desktop_guard();
    let config = format!(
        "[animations]\nenabled = true\nduration = {DURATION_MS}\neasing = \"linear\"\n\
         follow_system = false\n"
    );

    // Act
    let swap = swap_with("transitions-on", &config);

    // Assert -- each ends exactly in the other's slot, after several
    // places between the two, and never outside the box they make.
    assert_eq!(swap.after, [swap.before[1], swap.before[0]]);
    for (i, seen) in swap.seen.iter().enumerate() {
        let (from, to) = (swap.before[i], swap.after[i]);
        let mut between: Vec<Rect> = seen
            .iter()
            .copied()
            .filter(|r| ![from, to].contains(r))
            .collect();
        between.dedup();
        assert!(
            between.len() >= MIN_BETWEEN,
            "window {i} was seen at {} places between {from:?} and {to:?}: {seen:?}",
            between.len()
        );
        for rect in seen {
            assert!(
                within(*rect, from, to),
                "{rect:?} is outside {from:?} to {to:?}"
            );
        }
    }
}

#[test]
fn with_transitions_off_a_swap_jumps_straight_to_the_slots() {
    // Arrange -- the near miss: the same swap, with transitions off.
    let _guard = desktop_guard();

    // Act
    let swap = swap_with("transitions-off", "[animations]\nenabled = false\n");

    // Assert -- each window is only ever seen in one slot or the other.
    assert_eq!(swap.after, [swap.before[1], swap.before[0]]);
    for (i, seen) in swap.seen.iter().enumerate() {
        let slots = [swap.before[i], swap.after[i]];
        let stray: Vec<&Rect> = seen.iter().filter(|r| !slots.contains(r)).collect();
        assert!(
            stray.is_empty(),
            "window {i} was seen between its slots: {stray:?}"
        );
    }
}
