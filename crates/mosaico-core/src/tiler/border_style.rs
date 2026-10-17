//! How a focus border and the drop preview are drawn.

use crate::Rect;
use crate::config::Color;

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
