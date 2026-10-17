//! The pixels of an overlay's bitmap: a stroke with rounded corners, and
//! what is inside it, clear for a border and a translucent fill for the
//! drop preview.

use super::Color;

/// The pixel `color` gives at `alpha`, out of 255, as a layered window
/// takes it: each channel premultiplied by the alpha. Clear at 0.
pub(super) fn premultiplied(color: Color, alpha: u8) -> u32 {
    let scale = |channel: u8| u32::from(channel) * u32::from(alpha) / 255;
    (u32::from(alpha) << 24) | (scale(color.r) << 16) | (scale(color.g) << 8) | scale(color.b)
}

/// Paints `buf`, `w` by `h` pixels from the top row down: `stroke` along
/// the edge, `bw` pixels wide, with the outer corners rounded by `r`, and
/// `inside` within the stroke. What lies outside the rounded corners is
/// clear.
pub(super) fn paint(buf: &mut [u32], w: i32, h: i32, bw: i32, r: i32, stroke: u32, inside: u32) {
    let iw = w - 2 * bw;
    let ih = h - 2 * bw;
    let ir = (r - bw).max(0);
    for py in 0..h {
        for px in 0..w {
            let in_outer = in_rounded_rect(px, py, w, h, r);
            let in_inner = in_rounded_rect(px - bw, py - bw, iw, ih, ir);
            buf[(py * w + px) as usize] = match (in_outer, in_inner) {
                (_, true) => inside,
                (true, false) => stroke,
                (false, false) => 0,
            };
        }
    }
}

/// Tests if local coordinate `(lx, ly)` is inside a rounded rectangle
/// of size `(w, h)` with corner radius `r`.
pub(super) fn in_rounded_rect(lx: i32, ly: i32, w: i32, h: i32, r: i32) -> bool {
    if lx < 0 || lx >= w || ly < 0 || ly >= h {
        return false;
    }
    if r <= 0 {
        return true;
    }
    // Determine which corner zone the pixel falls in.
    let (cx, cy) = if lx < r && ly < r {
        (r, r) // top-left
    } else if lx >= w - r && ly < r {
        (w - r - 1, r) // top-right
    } else if lx < r && ly >= h - r {
        (r, h - r - 1) // bottom-left
    } else if lx >= w - r && ly >= h - r {
        (w - r - 1, h - r - 1) // bottom-right
    } else {
        return true; // not in a corner zone
    };
    let dx = lx - cx;
    let dy = ly - cy;
    dx * dx + dy * dy <= r * r
}

#[cfg(test)]
#[path = "pixels_tests.rs"]
mod tests;
