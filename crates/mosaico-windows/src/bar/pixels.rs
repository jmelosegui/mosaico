//! The pixels of a bar that takes the pointer.

use mosaico_core::bar::ItemRegion;

/// Gives every pixel left fully transparent inside a clickable item of
/// `regions` the least alpha there is, which no one can see, so that the
/// whole item takes the pointer: a layered window lets the pointer through
/// wherever its pixels are fully transparent. Every other pixel keeps the
/// alpha it was painted with, so the pointer goes through to the window
/// under the bar where the bar draws nothing.
///
/// Inside an item, those are the background of a bar with
/// `background_opacity = 0`, and the edges of glyphs that reach a pixel or
/// so past the text GDI measures, which GDI draws with no alpha. Their
/// colors are kept. `pixels` are rows `width` pixels wide, and the parts of
/// a region off them are left out.
pub(super) fn leave_no_hole(pixels: &mut [u32], width: i32, regions: &[ItemRegion]) {
    let width = usize::try_from(width).unwrap_or(0);
    let Some(height) = pixels.len().checked_div(width) else {
        return;
    };
    for rect in regions
        .iter()
        .filter(|region| region.clickable)
        .map(|r| r.rect)
    {
        let (left, right) = span(rect.x, rect.width, width);
        let (top, bottom) = span(rect.y, rect.height, height);
        for row in top..bottom {
            let line = &mut pixels[row * width + left..row * width + right];
            for pixel in line.iter_mut().filter(|pixel| **pixel >> 24 == 0) {
                *pixel |= 0x0100_0000;
            }
        }
    }
}

/// Where a region from `start`, `length` long, lies along a side of the
/// canvas `end` pixels long: its first pixel there and the one past its
/// last, both within the canvas.
fn span(start: i32, length: i32, end: usize) -> (usize, usize) {
    let within = |at: i32| usize::try_from(at).map_or(0, |at| at.min(end));
    let first = within(start);
    (first, within(start.saturating_add(length)).max(first))
}

#[cfg(test)]
#[path = "pixels_tests.rs"]
mod tests;
