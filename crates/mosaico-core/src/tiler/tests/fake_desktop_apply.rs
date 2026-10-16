//! How the fake desktop changes when an effect is applied to it, and
//! the order its windows are stacked in.

use crate::WindowId;
use crate::config::HidingBehaviour;

use super::super::super::Effect;
use super::FakeDesktop;

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

    /// Applies an effect the way a platform would.
    pub(crate) fn apply(&mut self, effect: &Effect) {
        match *effect {
            Effect::Place { window, rect, .. } => {
                // As `WindowControl::place` promises: no longer maximized,
                // but a minimized window stays minimized, and comes back
                // where it was before, as the contract allows.
                if let Some(w) = self.windows.get_mut(&window) {
                    if !w.minimized {
                        w.rect = rect;
                    }
                    w.maximized = false;
                }
            }
            Effect::Hide { window, how } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    match how {
                        HidingBehaviour::Cloak => w.cloaked = true,
                        HidingBehaviour::Hide => w.visible = false,
                        HidingBehaviour::Minimize => w.minimized = true,
                    }
                }
            }
            Effect::Show { window, .. } | Effect::Reveal { window } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.cloaked = false;
                    w.visible = true;
                    w.minimized = false;
                }
            }
            Effect::Minimize { window } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.minimized = true;
                }
            }
            Effect::Focus { window, .. } => {
                self.focused = Some(window);
                self.raise(window);
            }
            Effect::PutBehind { window, other } => {
                // As `WindowControl::put_behind`: nothing changes when
                // either window is gone.
                if window != other && self.stack.contains(&window) && self.stack.contains(&other) {
                    self.stack.retain(|&w| w != window);
                    let at = self.stack.iter().position(|&w| w == other);
                    self.stack.insert(at.map_or(0, |at| at + 1), window);
                }
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
            _ => {}
        }
    }
}
