//! Releasing what the coalescer holds, once input has gone quiet.

use super::super::{Effect, Timing};
use super::Coalescer;

impl Coalescer {
    /// Releases everything held, in the order to apply it: placements,
    /// then what waited for them, then the focus change and the restacks
    /// that wait for it.
    ///
    /// Each placement keeps whether it may animate, as the step that
    /// decided it marked it. A border is drawn around its window's held
    /// placement. A step that did not move the window works the rect out
    /// from the desktop, where the move has not happened yet (#133).
    pub fn take(&mut self) -> Vec<Effect> {
        let places = std::mem::take(&mut self.places);
        let mut out: Vec<Effect> = places
            .iter()
            .map(|&(window, rect, transition)| Effect::Place {
                window,
                rect,
                timing: Timing::Now,
                transition,
            })
            .collect();
        for mut effect in self.after.drain(..) {
            if let Effect::Border {
                window,
                style: Some(ref mut style),
            } = effect
                && let Some(&(_, rect, _)) = places.iter().find(|&&(w, ..)| w == window)
            {
                style.rect = rect;
            }
            out.push(effect);
        }
        out.extend(self.focus.take());
        out.append(&mut self.behind);
        out
    }
}

#[cfg(test)]
#[path = "tests/coalesce_transition_tests.rs"]
mod tests;
