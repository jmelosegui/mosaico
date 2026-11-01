//! Draws a widget's line icon into the bar's pixels, from the mask core
//! makes of its strokes.

use mosaico_core::bar::{BarIcon, mask};

use super::super::text::DrawCtx;
use crate::border::Color;

/// How an icon is sized beside text whose em is `em` pixels tall: as
/// tall as the em, with lines a ninth of it wide and a third of it
/// between the icon and the text, as the settings window previews it.
pub struct IconSize {
    pub size: i32,
    pub gap: i32,
    pub line: f32,
}

impl IconSize {
    pub fn for_em(em: i32) -> Self {
        let size = em.max(1);
        Self {
            size,
            gap: size / 3,
            line: (size as f32 / 9.0).max(1.0),
        }
    }
}

/// Draws `icon` `size` square at `x`, centered on the bar's height, in
/// `color_hex`.
pub fn draw(ctx: &mut DrawCtx, x: i32, icon: BarIcon, size: &IconSize, color_hex: &str) {
    let Some(color) = Color::from_hex(color_hex) else {
        return;
    };
    let side = size.size as usize;
    let coverage = mask(icon.strokes(), side, size.line);
    let top = (ctx.h - size.size) / 2;
    for (i, &alpha) in coverage.iter().enumerate() {
        let (px, py) = (x + (i % side) as i32, top + (i / side) as i32);
        if alpha == 0 || px < 0 || py < 0 || px >= ctx.w || py >= ctx.h {
            continue;
        }
        let idx = (py * ctx.w + px) as usize;
        if let Some(pixel) = ctx.buf.get_mut(idx) {
            *pixel = blend(*pixel, color, alpha);
        }
    }
}

/// `color`, covering `coverage` of 255 of the premultiplied pixel `dst`.
pub(super) fn blend(dst: u32, color: Color, coverage: u8) -> u32 {
    let a = u32::from(coverage);
    let keep = 255 - a;
    let channel = |shift: u32, src: u32| {
        let below = (dst >> shift) & 0xFF;
        ((src * a + below * keep + 127) / 255) << shift
    };
    channel(24, 255)
        | channel(16, u32::from(color.r))
        | channel(8, u32::from(color.g))
        | channel(0, u32::from(color.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color { r: 255, g: 0, b: 0 };

    #[test]
    fn a_fully_covered_pixel_takes_the_color() {
        assert_eq!(blend(0xFF31_3244, RED, 255), 0xFFFF_0000);
    }

    #[test]
    fn an_uncovered_pixel_is_left_alone() {
        assert_eq!(blend(0xFF31_3244, RED, 0), 0xFF31_3244);
    }

    #[test]
    fn a_half_covered_pixel_mixes_the_color_with_what_is_below() {
        assert_eq!(blend(0xFF00_0000, RED, 128), 0xFF80_0000);
        assert_eq!(blend(0x0000_0000, RED, 128), 0x8080_0000);
    }

    #[test]
    fn an_icon_is_as_tall_as_the_em_with_lines_in_proportion() {
        let size = IconSize::for_em(18);

        assert_eq!((size.size, size.gap, size.line), (18, 6, 2.0));
    }
}
