//! Bars drawn by the fake renderer, and the pointer on them.

use std::time::{Duration, Instant};

use crate::Action;
use crate::action::Origin;
use crate::bar::fakes::layout::{EDGE, INSET, LABEL, LABEL_GAP};
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::bar::{BarId, BarManager, BarState, Button, PointerEvent};
use crate::config::BarConfig;

use super::super::tests::{left, right, start};

/// A manager with `config` on two monitors, drawn once.
pub(super) fn drawn(config: BarConfig, renderer: &FakeRenderer) -> BarManager {
    drawn_with(
        config,
        &[BarState::default(), BarState::default()],
        renderer,
    )
}

/// A manager with `config` on two monitors, drawn with `states`.
pub(super) fn drawn_with(
    config: BarConfig,
    states: &[BarState],
    renderer: &FakeRenderer,
) -> BarManager {
    let mut manager = start(config, &[left(), right()], renderer, &FakeSystem::default());
    manager.update(states);
    manager
}

/// Monitors showing the workspaces at `active`, from 0, of 8.
pub(super) fn showing(active: &[usize]) -> Vec<BarState> {
    active
        .iter()
        .map(|&active_workspace| BarState {
            active_workspace,
            ..BarState::default()
        })
        .collect()
}

/// The bar the manager made on `monitor` last.
pub(super) fn bar_on(renderer: &FakeRenderer, monitor: usize) -> BarId {
    let ids = renderer.ids();
    *ids.iter()
        .rev()
        .find(|id| id.monitor == monitor)
        .expect("a bar on that monitor")
}

/// The middle of the label of workspace `index`, from 0, on the default
/// bar, whose first widget is the workspaces one.
pub(super) fn label(index: usize) -> i32 {
    EDGE + INSET + index as i32 * (LABEL + LABEL_GAP) + LABEL / 2
}

pub(super) fn press(button: Button, x: i32) -> PointerEvent {
    PointerEvent::Press { button, x, y: 5 }
}

pub(super) fn release(button: Button, x: i32) -> PointerEvent {
    PointerEvent::Release { button, x, y: 5 }
}

/// What a press at `down` and a release at `up` with the left button on
/// `bar` ask for.
pub(super) fn click(
    manager: &mut BarManager,
    bar: BarId,
    down: i32,
    up: i32,
) -> Option<(Action, Origin)> {
    let now = Instant::now();
    manager.pointer(bar, press(Button::Left, down), now);
    manager.pointer(bar, release(Button::Left, up), now)
}

/// Turns the wheel by each of `notches` over `bar`, 10 ms apart from
/// `start`, and returns when the last turn came.
pub(super) fn turn(
    manager: &mut BarManager,
    bar: BarId,
    notches: &[f32],
    start: Instant,
) -> Instant {
    let mut at = start;
    for (i, &notches) in notches.iter().enumerate() {
        at = start + Duration::from_millis(10) * i as u32;
        let event = PointerEvent::Wheel {
            notches,
            x: 5,
            y: 5,
        };
        manager.pointer(bar, event, at);
    }
    at
}
