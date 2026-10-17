//! The bar's scenarios: a click on a workspace label of the focused
//! monitor's bar and of another monitor's, and a burst of wheel turns,
//! which the bar manager adds up to one switch.

use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::action::Origin;
use crate::bar::fakes::{FakeRenderer, FakeSystem, NoPointer};
use crate::bar::{BarManager, PointerEvent, WHEEL_QUIET};
use crate::config::{BarConfig, Theme};
use crate::{Action, Direction, Rect};

use super::super::fake_platform::FakePlatform;
use super::super::manager_support::{start, still};
use super::super::metrics_support::{Manager, Measured, on};
use super::super::{MON_H, MON_W};

/// Two windows on each of two monitors, the first of each sent to
/// workspace 4, both monitors back on workspace 1, and focus on monitor 0.
fn fourth_workspaces() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let mut manager = start(&platform, None, &still());
    let arrange = [
        Action::SendToWorkspace(4),
        Action::GoToWorkspace(1),
        Action::Focus(Direction::Right),
        Action::SendToWorkspace(4),
        Action::GoToWorkspace(1),
        Action::Focus(Direction::Left),
    ];
    for action in &arrange {
        manager.handle_action(action);
    }
    manager.flush_pending();
    (platform, manager)
}

/// A click on the label of workspace 4 on the bar of `monitor`.
fn click(monitor: usize) -> Measured {
    on(fourth_workspaces(), |m| {
        m.handle_action_from(&Action::GoToWorkspace(4), Origin::Bar(monitor));
    })
}

/// A click on the focused monitor's bar.
pub(super) fn click_on_the_focused_monitor() -> Measured {
    click(0)
}

/// A click on the bar of the monitor without focus, which takes it.
pub(super) fn click_on_another_monitor() -> Measured {
    click(1)
}

/// Three notches turned in a burst over monitor 0's bar, which its bar
/// manager adds up and, once the wheel goes quiet, switches with once,
/// three workspaces on.
pub(super) fn scroll_burst() -> Measured {
    on(fourth_workspaces(), |m| {
        let renderer = FakeRenderer::default();
        let mut bars = drawn_bars(m, &renderer);
        let first = renderer.ids()[0];
        let start = Instant::now();
        for i in 0..3 {
            let turn = PointerEvent::Wheel {
                notches: 1.0,
                x: 5,
                y: 5,
            };
            bars.pointer(first, turn, start + Duration::from_millis(10) * i);
        }
        for (action, origin) in bars.take_wheel(start + Duration::from_millis(20) + WHEEL_QUIET) {
            m.handle_action_from(&action, origin);
        }
    })
}

/// A bar on each monitor of `manager`, drawn by `renderer` with what the
/// manager has on screen.
fn drawn_bars(manager: &Manager, renderer: &FakeRenderer) -> BarManager {
    let areas = vec![
        Rect::new(0, 0, MON_W, MON_H),
        Rect::new(MON_W, 0, MON_W, MON_H),
    ];
    let system = Box::new(FakeSystem::default());
    let config = BarConfig::default();
    let mut bars = BarManager::new(
        renderer.boxed(),
        system,
        Rc::new(NoPointer),
        config,
        areas,
        Theme::default(),
    );
    bars.update(&manager.bar_states("", false));
    bars
}
