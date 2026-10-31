//! How a focus border and the drop preview are drawn.

use crate::config::Color;
use crate::{Rect, WindowId};

/// How a focus border is drawn. Where the stroke goes is part of the
/// [`Decorations`](crate::Decorations) contract.
#[derive(Debug, Clone, PartialEq)]
pub struct BorderStyle {
    /// The window's visible bounds, which the border surrounds.
    pub rect: Rect,
    /// The stroke color. A configured value that is not a color is
    /// already replaced by
    /// [`BorderColors::FALLBACK`](crate::config::BorderColors::FALLBACK).
    pub color: Color,
    /// Stroke width in pixels.
    pub width: i32,
    /// Radius of the border's outer corners, in pixels.
    pub radius: i32,
    /// The window the border goes directly behind instead of its own:
    /// the backmost floating window, for a focused tile that the floating
    /// windows of its workspace are kept in front of. The border then
    /// stays behind them however the tile and the border land in the
    /// stacking order.
    pub behind: Option<WindowId>,
}

/// How the drop preview is drawn: an outline over a fill that lets what
/// is under it show through, as
/// [`Decorations::show_preview`](crate::Decorations::show_preview) says.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewStyle {
    /// The color of the outline and of the fill, the focused border's.
    pub color: Color,
    /// Outline width in pixels.
    pub width: i32,
    /// Radius of the outer corners, in pixels.
    pub radius: i32,
}
