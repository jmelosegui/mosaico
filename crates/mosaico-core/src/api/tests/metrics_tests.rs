//! What the bars are sent must not grow unnoticed: each scenario's
//! messages and bytes are checked against the committed baseline in
//! `metrics/api.toml`.

use std::path::Path;
use std::time::Duration;

use mosaico_metrics::{Baseline, Better, Report};
use serde_json::Value;

use crate::StableMonitorId;

use super::State;
use super::examples_tests::path;
use super::fake_pipe::FakeBar;
use super::session_world::{HELLO, World};

/// What records a new baseline, for failure messages to suggest.
const BLESS_WITH: &str = "cargo xtask metrics --bless";

/// A `hello` for the focus only, as a bar showing the focused window sends.
const FOCUS_ONLY: &str = r#"{"hello":{"api":[1],"topics":["focus"]}}"#;

/// How long a bar waits for more once it got what a change should send.
const QUIET: Duration = Duration::from_millis(100);

/// What a bar was sent: messages, and their bytes as lines.
#[derive(Default)]
struct Sent {
    messages: u64,
    bytes: u64,
}

impl Sent {
    fn add(&mut self, message: &Value) {
        let line = serde_json::to_string(message).unwrap_or_default();
        self.messages += 1;
        self.bytes += line.len() as u64 + 1;
    }

    /// The `expected` messages `bar` is sent, each given time to come
    /// however busy the machine, then whatever else comes until it goes
    /// quiet.
    fn read(bar: &mut FakeBar, expected: usize) -> Self {
        let mut sent = Self::default();
        for message in (0..expected).map_while(|_| bar.recv()) {
            sent.add(&message);
        }
        while let Some(message) = bar.recv_within(QUIET) {
            sent.add(&message);
        }
        sent
    }
}

/// The documented example state, with a second monitor that can leave.
fn example() -> State {
    let text = std::fs::read_to_string(path("state.json")).unwrap_or_default();
    let mut state: State = serde_json::from_str(&text).expect("the example state reads");
    let mut second = state.monitors[0].clone();
    second.id = StableMonitorId::new("m-7b1d04");
    second.index = 1;
    state.monitors.push(second);
    state
}

/// What a bar saying `hello` is sent on connecting.
fn connect(hello: &str) -> Sent {
    let world = World::new();
    world.hub.publish(&example());
    let mut bar = world.listener.connect();
    bar.send(hello);
    Sent::read(&mut bar, 2)
}

/// What a bar saying `hello` is sent once the example state turns into
/// what `change` makes of it, which sends `expected` messages.
fn after(hello: &str, change: fn(&mut State), expected: usize) -> Sent {
    let world = World::new();
    let mut state = example();
    world.hub.publish(&state);
    let (mut bar, _, _) = world.open(hello);
    change(&mut state);
    world.hub.publish(&state);
    Sent::read(&mut bar, expected)
}

fn switch_workspace(state: &mut State) {
    state.monitors[0].active_workspace = 2;
    state.focus.workspace = Some(2);
    state.focus.window = None;
}

fn retitle_focused_window(state: &mut State) {
    if let Some(window) = &mut state.focus.window {
        window.title = "plan.md - mosaico".to_string();
    }
}

fn disconnect_monitor(state: &mut State) {
    state.monitors.pop();
}

fn nothing(_: &mut State) {}

/// Every scenario's messages and bytes.
fn measure() -> Report {
    let scenarios = [
        ("connect", connect(HELLO)),
        ("connect_focus_only", connect(FOCUS_ONLY)),
        ("switch_workspace", after(HELLO, switch_workspace, 1)),
        (
            "switch_workspace_focus_only",
            after(FOCUS_ONLY, switch_workspace, 1),
        ),
        (
            "retitle_focused_window",
            after(HELLO, retitle_focused_window, 1),
        ),
        ("disconnect_monitor", after(HELLO, disconnect_monitor, 1)),
        ("publish_unchanged", after(HELLO, nothing, 0)),
    ];
    let mut report = Report::new();
    for (name, sent) in scenarios {
        report.set(name, "messages", sent.messages, Better::Lower);
        report.set(name, "bytes", sent.bytes, Better::Lower);
    }
    report
}

#[test]
fn what_the_bars_are_sent_matches_the_baseline() {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2);
    let path = root
        .expect("the crate is in crates/")
        .join("metrics")
        .join("api.toml");
    let bless = std::env::var_os("MOSAICO_BLESS").is_some();

    // Act
    let report = measure();

    // Assert
    match Baseline::new(&path, BLESS_WITH).check(&report, bless) {
        Ok(changes) => {
            for change in changes {
                eprintln!("recorded {change}");
            }
        }
        Err(failure) => panic!("{failure}"),
    }
}
