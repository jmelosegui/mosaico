//! Reading the order of a step's effects, for the tests of #28.

use super::*;

/// The first effect of a step that moves a window or draws a border.
pub(super) fn first_move_or_border(effects: &[Effect]) -> Option<String> {
    effects.iter().find_map(|e| match *e {
        Effect::PrePlace { window, .. } => Some(format!("post move {}", window.raw())),
        Effect::Place { window, .. } => Some(format!("place {}", window.raw())),
        Effect::Border { window, .. } => Some(format!("border {}", window.raw())),
        _ => None,
    })
}

/// Where a step's move that does not wait sends `window`, if it has one.
pub(super) fn posted_move(effects: &[Effect], window: u64) -> Option<Rect> {
    effects.iter().find_map(|e| match *e {
        Effect::PrePlace { window: w, rect } if w == wid(window) => Some(rect),
        _ => None,
    })
}

/// What a step does other than drawing borders, in order.
pub(super) fn moves(effects: &[Effect]) -> Vec<String> {
    effects
        .iter()
        .filter(|e| !matches!(e, Effect::Border { .. }))
        .map(|e| match *e {
            Effect::PrePlace { window, .. } => format!("post move {}", window.raw()),
            Effect::Place { window, .. } => format!("place {}", window.raw()),
            Effect::Decorate { window, .. } => format!("decorate {}", window.raw()),
            Effect::Focus { window, .. } => format!("focus {}", window.raw()),
            Effect::Show { window, .. } => format!("show {}", window.raw()),
            Effect::Hide { window, .. } => format!("hide {}", window.raw()),
            Effect::DropBorder { window } => format!("drop border {}", window.raw()),
            ref other => format!("{other:?}"),
        })
        .collect()
}
