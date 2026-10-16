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
pub(super) fn dedup(events: &mut Vec<WindowEvent>) {
    let mut last = HashMap::new();
    for (i, event) in events.iter().enumerate() {
        if let Some(key) = repeated(event) {
            last.insert(key, i);
        }
    }

    let mut i = 0;
    events.retain(|event| {
        let keep = repeated(event).is_none_or(|key| last.get(&key) == Some(&i));
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

#[cfg(test)]
#[path = "tests/dedup_tests.rs"]
mod tests;
