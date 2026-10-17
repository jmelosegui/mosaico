//! Labels and a pill drawn on a bar, and the pointer clicking them.

use crate::Rect;
use crate::bar::{BarSide, WidgetRef};

use super::super::{BarTarget, Button, ItemRegion, PointerEvent, Press};

/// Labels of workspaces 0 and 1 at 0..20 and 30..50, and a pill at
/// 60..160, all 30 pixels tall.
pub(super) fn drawn() -> Vec<ItemRegion> {
    let label = |workspace: usize, x: i32| ItemRegion {
        target: BarTarget::Workspace(workspace),
        clickable: true,
        rect: Rect::new(x, 0, 20, 30),
    };
    vec![
        label(0, 0),
        label(1, 30),
        ItemRegion {
            target: BarTarget::Widget(WidgetRef {
                side: BarSide::Left,
                index: 1,
            }),
            clickable: false,
            rect: Rect::new(60, 0, 100, 30),
        },
    ]
}

pub(super) fn press(x: i32) -> PointerEvent {
    PointerEvent::Press {
        button: Button::Left,
        x,
        y: 10,
    }
}

pub(super) fn release(x: i32) -> PointerEvent {
    PointerEvent::Release {
        button: Button::Left,
        x,
        y: 10,
    }
}

/// What `events` click, in order, on a bar that drew `regions`.
pub(super) fn clicks(events: &[PointerEvent], regions: &[ItemRegion]) -> Vec<(Button, BarTarget)> {
    let mut state = Press::default();
    events
        .iter()
        .filter_map(|&event| state.take(event, regions))
        .collect()
}
