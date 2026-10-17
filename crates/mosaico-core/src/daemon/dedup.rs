//! Drops the window events of a batch that say nothing new.

use std::collections::HashMap;

use crate::{WindowEvent, WindowId};

/// Changes of which only the latest one per window matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Repeated {
    Location,
    Title,
}

/// Collapses repeated position and title changes in a batch.
///
/// Windows report position and title changes many times per second,
/// during an animation for example, and an earlier one says nothing the
/// latest does not. Each window keeps its last `LocationChanged` and its
/// last `TitleChanged`, where that one arrived, which can turn hundreds
/// of events into a handful. Every other event is kept, in order:
/// handling it twice is not the same as handling it once.
///
/// The start and the end of a drag are barriers: a change is only
/// collapsed with those on the same side of both. Where a window is
/// while the user drags it is what the drag is followed by, and a
/// change from before the drag or after it says nothing about that.
pub(super) fn dedup(events: &mut Vec<WindowEvent>) {
    let mut last = HashMap::new();
    let mut span = 0;
    for (i, event) in events.iter().enumerate() {
        span += usize::from(is_barrier(event));
        if let Some(key) = repeated(event) {
            last.insert((key, span), i);
        }
    }

    let mut i = 0;
    let mut span = 0;
    events.retain(|event| {
        span += usize::from(is_barrier(event));
        let keep = repeated(event).is_none_or(|key| last.get(&(key, span)) == Some(&i));
        i += 1;
        keep
    });
}

/// The window and kind of a change only the latest of which matters.
fn repeated(event: &WindowEvent) -> Option<(WindowId, Repeated)> {
    match *event {
        WindowEvent::LocationChanged { window } => Some((window, Repeated::Location)),
        WindowEvent::TitleChanged { window } => Some((window, Repeated::Title)),
        _ => None,
    }
}

/// Whether no change is collapsed across `event`: the start or the end
/// of a drag, whichever window it is.
fn is_barrier(event: &WindowEvent) -> bool {
    matches!(
        event,
        WindowEvent::MoveStarted { .. } | WindowEvent::Moved { .. }
    )
}

#[cfg(test)]
#[path = "tests/dedup_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/dedup_drag_tests.rs"]
mod drag_tests;
