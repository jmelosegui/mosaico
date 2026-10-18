//! The preview of where a window the user drags lands.
//!
//! While the user moves a tiled window with `[mouse] drag_to_swap` on,
//! each report that it moved works out where it would land if dropped
//! now, with the pointer where it is, by the rule the drop follows
//! (`drag_drop.rs`), and the preview shows the slot it would get: its
//! own, that of the tile under the pointer, or the one it would get on
//! another monitor once put before the tile under the pointer, which in
//! BSP is not that tile, and where a workspace in monocle is laid out as
//! if it were tiled. A drop that would be handled as before drops moved
//! windows shows none, as a resize or a window back where it started. The
//! preview is drawn again only when where it shows changes, and hidden as
//! the drag ends or is interrupted.
//! A focus change that leaves the drag going shows it again, which puts
//! it back just behind the window the user drags. Without reports while
//! the window is dragged, as with window contents hidden while dragging,
//! there is no preview, and the drop decides from where the window and
//! the pointer are as it ends.

use crate::Rect;
use crate::config::{BorderColors, Color};

use super::super::{Effect, Plan, PreviewStyle, Tiler};
use super::Drag;
use super::target::Target;

impl Tiler {
    /// Shows the preview where the window the user drags would land if
    /// dropped now, or hides it when it would land nowhere.
    pub(in crate::tiler) fn preview_drop(&mut self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.as_ref().filter(|d| d.decides) else {
            return;
        };
        let lands = self.landing(plan, drag);
        if lands == drag.preview {
            return;
        }
        let window = drag.window;
        match lands {
            Some(rect) => {
                let style = self.preview_style();
                plan.emit(Effect::ShowPreview {
                    window,
                    rect,
                    style,
                });
            }
            None => plan.emit(Effect::HidePreview),
        }
        if let Some(drag) = self.mouse.drag.as_mut() {
            drag.preview = lands;
        }
    }

    /// Shows the preview again where it shows, as a window came to the
    /// front: just behind the window the user drags, in front of the one
    /// that came.
    pub(in crate::tiler) fn restack_preview(&self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.as_ref() else {
            return;
        };
        if let Some(rect) = drag.preview {
            let (window, style) = (drag.window, self.preview_style());
            plan.emit(Effect::ShowPreview {
                window,
                rect,
                style,
            });
        }
    }

    /// Where the window `drag` is about would be placed if dropped now,
    /// with the pointer where the desktop has it.
    fn landing(&self, plan: &Plan, drag: &Drag) -> Option<Rect> {
        match self.drop_target(plan, drag, plan.cursor())? {
            Target::Back => self.slot_of(drag.monitor, drag.window),
            Target::Swap(other) => self.slot_of(drag.monitor, other),
            Target::Join { monitor, before } => self.slot_joining(monitor, before, drag.window),
        }
    }

    /// How the preview is drawn: in the focused border's color, as wide
    /// and as round as the borders.
    fn preview_style(&self) -> PreviewStyle {
        let borders = &self.border_config;
        PreviewStyle {
            color: Color::from_hex(&borders.colors.focused).unwrap_or(BorderColors::FALLBACK),
            width: borders.width,
            radius: borders.corner_style.border_radius(),
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_preview_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_preview_cases_tests.rs"]
mod cases_tests;

#[cfg(test)]
#[path = "tests/drag_preview_end_tests.rs"]
mod end_tests;

#[cfg(test)]
#[path = "tests/drag_preview_size_tests.rs"]
mod size_tests;

#[cfg(test)]
#[path = "tests/drag_preview_bar_tests.rs"]
mod bar_tests;
