//! The borders of moving windows.
//!
//! A border is drawn around its window's rect, and drawing one can take
//! tens of milliseconds for a large window, too long for a frame. So a
//! `Border` effect for a moving window only replaces the style its motion
//! draws once the window is in place, at the rect it ends at. Hiding a
//! border is cheap and happens at once. A window that changes size on its
//! way has its border hidden until then, since the border could not
//! follow it.

use crate::{Decorations, Rect, WindowId};

use super::super::{BorderStyle, Effect, apply_decoration_effect};
use super::Applier;

/// What the border of a moving window does until its motion ends.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct MotionBorder {
    /// The style to draw once the window is in place: the last one a
    /// `Border` effect gave, unless it hid the border. `None` too while no
    /// effect came, which leaves the border as it is.
    style: Option<BorderStyle>,
    /// Whether the window changes size on its way.
    resizes: bool,
    /// Whether the border has been hidden for the motion.
    hidden: bool,
}

impl Applier {
    /// Notes that `window` moves on from `from` to `to`, so a border that
    /// could not follow it is hidden.
    pub(super) fn border_moves(&mut self, window: WindowId, from: Rect, to: Rect) {
        let border = self.transitions.borders.entry(window).or_default();
        border.resizes |= (from.width, from.height) != (to.width, to.height);
    }

    /// Carries out a `Border` effect for `window`, which is moving: the
    /// style replaces the one drawn once the window is in place, and a
    /// border that is to be hidden, or that cannot follow its window, is
    /// hidden at once.
    pub(super) fn border_of_moving(
        &mut self,
        window: WindowId,
        style: &Option<BorderStyle>,
        decorations: &mut dyn Decorations,
    ) {
        let border = self.transitions.borders.entry(window).or_default();
        border.style = style.clone();
        if (style.is_none() || border.resizes) && !border.hidden {
            border.hidden = true;
            let hide = Effect::Border {
                window,
                style: None,
            };
            apply_decoration_effect(&hide, decorations);
        }
    }

    /// Draws the border `window` had kept for its motion, around `rect`,
    /// where the window is now in place, and forgets it.
    pub(super) fn draw_kept_border(
        &mut self,
        window: WindowId,
        rect: Rect,
        decorations: &mut dyn Decorations,
    ) {
        let kept = self.transitions.borders.remove(&window);
        if let Some(mut style) = kept.and_then(|border| border.style) {
            style.rect = rect;
            let draw = Effect::Border {
                window,
                style: Some(style),
            };
            apply_decoration_effect(&draw, decorations);
        }
    }

    /// Forgets the border `window` had kept for its motion: what the tiler
    /// decides for it next is drawn as it comes.
    pub(super) fn forget_kept_border(&mut self, window: WindowId) {
        self.transitions.borders.remove(&window);
    }
}
