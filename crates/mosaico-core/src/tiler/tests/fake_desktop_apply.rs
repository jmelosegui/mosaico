//! How the fake desktop changes when an effect is applied to it, and
//! the order its windows are stacked in.

use std::time::Instant;

use crate::WindowId;
use crate::config::HidingBehaviour;

use super::super::super::Effect;
use super::FakeDesktop;
use super::frames::Queued;

impl FakeDesktop {
    /// Brings a window to the front of the stacking order.
    pub(crate) fn raise(&mut self, window: WindowId) {
        self.stack.retain(|&w| w != window);
        self.stack.insert(0, window);
    }

    /// Whether `front` is in front of `back` in the stacking order.
    pub(crate) fn in_front(&self, front: WindowId, back: WindowId) -> bool {
        let at = |window: WindowId| self.stack.iter().position(|&w| w == window);
        matches!((at(front), at(back)), (Some(f), Some(b)) if f < b)
    }

    /// Puts `window` right behind `other` in the stacking order, as
    /// `WindowControl::put_behind` does: nothing changes when either
    /// window is gone.
    pub(super) fn put_behind(&mut self, window: WindowId, other: WindowId) {
        if window != other && self.stack.contains(&window) && self.stack.contains(&other) {
            self.stack.retain(|&w| w != window);
            let at = self.stack.iter().position(|&w| w == other);
            self.stack.insert(at.map_or(0, |at| at + 1), window);
        }
    }

    /// Applies an effect at `now` the way a platform would: a placement,
    /// a change of show state and one of the stacking order at once,
    /// unless something may be queued for the window, behind which they
    /// are queued too.
    pub(crate) fn apply(&mut self, effect: &Effect, now: Instant) {
        match *effect {
            Effect::Place { window, rect, .. } => self.change(window, Queued::Place(rect), now),
            Effect::Hide { window, how } => match how {
                HidingBehaviour::Cloak => {
                    if let Some(w) = self.windows.get_mut(&window) {
                        w.cloaked = true;
                    }
                }
                HidingBehaviour::Hide => self.change(window, Queued::Hide, now),
                HidingBehaviour::Minimize => self.change(window, Queued::Minimize, now),
            },
            Effect::Show { window, how } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.cloaked = false;
                    if how == HidingBehaviour::Cloak {
                        w.visible = true;
                        w.minimized = false;
                    }
                }
                if how != HidingBehaviour::Cloak {
                    self.change(window, Queued::Show, now);
                }
            }
            Effect::Reveal { window } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.cloaked = false;
                }
                self.change(window, Queued::Reveal, now);
            }
            Effect::Minimize { window } => self.change(window, Queued::Minimize, now),
            Effect::Focus { window, .. } => {
                // As on the platform, the window that has focus already is
                // not reordered by being focused again.
                if self.focused != Some(window) {
                    self.raise(window);
                }
                self.focused = Some(window);
            }
            Effect::PutBehind { window, other } => {
                self.change(window, Queued::PutBehind(other), now);
            }
            Effect::Border {
                window,
                style: Some(_),
            } => {
                self.borders.insert(window, true);
            }
            Effect::Border {
                window,
                style: None,
            } => {
                if let Some(showing) = self.borders.get_mut(&window) {
                    *showing = false;
                }
            }
            Effect::DropBorder { window } => {
                self.borders.remove(&window);
            }
            Effect::ShowPreview { rect, .. } => self.preview = Some(rect),
            Effect::HidePreview => self.preview = None,
            _ => {}
        }
    }
}
