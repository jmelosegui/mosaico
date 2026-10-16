//! When the pointer has come to rest on a window.

use crate::WindowId;

use super::REST_POLLS;

/// Decides when the pointer has come to rest on a window.
///
/// It is told the window under the pointer at each poll. A window is
/// reported once the pointer has been on it for [`REST_POLLS`] polls in
/// a row, and only once until the pointer moves to another window. A
/// poll that finds no window is skipped: it neither counts towards a
/// rest nor breaks one.
#[derive(Debug, Default)]
pub(super) struct HoverTracker {
    /// The window the pointer was last found on, and for how many polls
    /// in a row.
    resting: Option<(WindowId, u8)>,
    /// Whether that window has been reported.
    reported: bool,
}

impl HoverTracker {
    /// Takes the window one poll found under the pointer, and returns
    /// the window to report, if any.
    pub(super) fn observe(&mut self, under: Option<WindowId>) -> Option<WindowId> {
        let window = under?;
        let polls = match &mut self.resting {
            Some((last, polls)) if *last == window => {
                *polls = polls.saturating_add(1);
                *polls
            }
            _ => {
                self.resting = Some((window, 1));
                self.reported = false;
                1
            }
        };
        if polls < REST_POLLS || self.reported {
            return None;
        }
        self.reported = true;
        Some(window)
    }
}

#[cfg(test)]
#[path = "tests/rest_tests.rs"]
mod tests;
