//! What floating windows ask of the platform: floating one and tiling it
//! again, moving and resizing it with the keys, and one the rules float
//! opening.

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};
use crate::tiler::tests::metrics_support::{Manager, Measured, burst, four, on, windows};
use crate::{Action, WindowEvent};

/// Four windows on one monitor, with window 2 focused and floated.
fn floated() -> (FakePlatform, Manager) {
    let (platform, mut manager) = four();
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    manager.handle_action(&Action::ToggleFloat);
    manager.flush_pending();
    (platform, manager)
}

/// Window 2 of four, focused, floats and is tiled again.
pub(super) fn on_and_off() -> Measured {
    let (platform, mut manager) = four();
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    manager.flush_pending();
    on((platform, manager), |manager| {
        manager.handle_action(&Action::ToggleFloat);
        manager.handle_action(&Action::ToggleFloat);
    })
}

/// The floating window is nudged right and left nine times.
pub(super) fn nudge_burst() -> Measured {
    on(floated(), |manager| {
        for action in burst(Action::Move) {
            manager.handle_action(&action);
        }
    })
}

/// The floating window grows wider once.
pub(super) fn grow() -> Measured {
    on(floated(), |manager| {
        manager.handle_action(&Action::GrowWidth)
    })
}

/// Window 99, which the rules float, opens next to four tiles.
pub(super) fn open_by_rule() -> Measured {
    let mut open = windows(4);
    open.push((99, 0));
    let platform = FakePlatform::new(1, &open);
    platform.edit(99, |w| {
        w.visible = false;
        if let Some(facts) = w.facts.as_mut() {
            facts.class = "Tool".into();
        }
    });
    let mut manager = start(&platform, None, &still());
    manager.reload_rules(vec![WindowRule {
        match_class: Some("Tool".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Float,
    }]);
    manager.flush_pending();
    platform.edit(99, |w| w.visible = true);
    on((platform, manager), |manager| {
        manager.handle_event(&WindowEvent::Created { window: wid(99) })
    })
}
