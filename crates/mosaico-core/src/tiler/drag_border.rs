//! The border of the window the user drags.
//!
//! It is left as it is until a report finds the window away from where it
//! started, and hidden from then until the drag ends, as it would only
//! show where the window was. A click on the title bar, which the platform
//! reports as a drag that moves nothing, leaves it drawn.

use crate::WindowId;

use super::super::{Effect, Plan, Tiler};

impl Tiler {
    /// Whether the border of `window` is hidden for its drag: from the
    /// report that found it away from where it started to the drag's end.
    pub(in crate::tiler) fn hides_border(&self, window: WindowId) -> bool {
        let drag = self.mouse.drag.as_ref();
        drag.is_some_and(|d| d.window == window && d.left_start)
    }

    /// Takes a report of where the window the user drags is: the first that
    /// finds it away from where it started hides its border.
    pub(in crate::tiler) fn drag_reported(&mut self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.as_mut() else {
            return;
        };
        if drag.left_start || plan.rect(drag.window) == Some(drag.start) {
            return;
        }
        drag.left_start = true;
        let window = drag.window;
        if self.bordered.contains(&window) {
            plan.emit(Effect::Border {
                window,
                style: None,
            });
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_border_tests.rs"]
mod tests;
