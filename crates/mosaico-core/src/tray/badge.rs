//! The paused variant of the icon: the running icon dimmed, with a red
//! pause badge in the corner, like the bar's paused widget.

/// The color of the pause badge: the red of the default theme.
pub const PAUSED_BADGE: [u8; 3] = [0xF3, 0x8B, 0xA8];

/// How much of the badge's square side its radius takes.
const BADGE_RADIUS: f32 = 0.3;

/// Samples per pixel side when measuring how much of a pixel the badge
/// covers, so its edges are smooth at small sizes.
const SAMPLES: usize = 4;

/// Turns the `size` by `size` icon in `pixels`, rows top to bottom in
/// RGBA with straight alpha, into its paused variant: faded and mostly
/// gray, with a red disc holding two white bars in the bottom right
/// corner.
///
/// Does nothing when `pixels` does not hold `size` by `size` pixels.
pub fn paint_paused(pixels: &mut [[u8; 4]], size: usize) {
    if size == 0 || pixels.len() != size * size {
        return;
    }
    for pixel in pixels.iter_mut() {
        fade(pixel);
    }
    let badge = Badge::new(size);
    for (i, pixel) in pixels.iter_mut().enumerate() {
        let (disc, bars) = badge.coverage(i % size, i / size);
        over(pixel, PAUSED_BADGE, disc);
        over(pixel, [0xFF, 0xFF, 0xFF], bars);
    }
}

/// Grays a pixel most of the way and makes it more transparent.
fn fade(pixel: &mut [u8; 4]) {
    let [r, g, b, a] = pixel.map(f32::from);
    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
    let mix = |c: f32| channel(c + (gray - c) * 0.7);
    *pixel = [mix(r), mix(g), mix(b), channel(a * 0.55)];
}

/// Paints `color` over `pixel`, covering `coverage` of it.
fn over(pixel: &mut [u8; 4], color: [u8; 3], coverage: f32) {
    if coverage <= 0.0 {
        return;
    }
    let below = f32::from(pixel[3]) / 255.0;
    let alpha = coverage + below * (1.0 - coverage);
    for (c, top) in pixel.iter_mut().zip(color) {
        let mixed = f32::from(top) * coverage + f32::from(*c) * below * (1.0 - coverage);
        *c = channel(mixed / alpha);
    }
    pixel[3] = channel(alpha * 255.0);
}

fn channel(value: f32) -> u8 {
    // Clamped to the range of a channel first, so the cast keeps it.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let c = value.round().clamp(0.0, 255.0) as u8;
    c
}

/// Where the badge is on an icon of a given size.
struct Badge {
    center: f32,
    radius: f32,
    bar_width: f32,
    bar_height: f32,
    bar_gap: f32,
}

impl Badge {
    fn new(size: usize) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let side = size as f32;
        let radius = side * BADGE_RADIUS;
        Self {
            center: side - radius,
            radius,
            bar_width: radius * 0.34,
            bar_height: radius * 1.0,
            bar_gap: radius * 0.3,
        }
    }

    /// How much of the pixel at `x`, `y` the disc covers, and how much
    /// the bars do, from 0 to 1 each.
    fn coverage(&self, x: usize, y: usize) -> (f32, f32) {
        let (mut disc, mut bars) = (0, 0);
        for sy in 0..SAMPLES {
            for sx in 0..SAMPLES {
                let px = sample(x, sx) - self.center;
                let py = sample(y, sy) - self.center;
                if px * px + py * py <= self.radius * self.radius {
                    disc += 1;
                    if self.on_bar(px, py) {
                        bars += 1;
                    }
                }
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let total = (SAMPLES * SAMPLES) as f32;
        #[allow(clippy::cast_precision_loss)]
        let ratio = |n: usize| n as f32 / total;
        (ratio(disc), ratio(bars))
    }

    /// Whether a point, relative to the center, is on either bar.
    fn on_bar(&self, x: f32, y: f32) -> bool {
        let inner = self.bar_gap / 2.0;
        let outer = inner + self.bar_width;
        let across = x.abs();
        y.abs() <= self.bar_height / 2.0 && across >= inner && across <= outer
    }
}

/// The position of sample `s` inside pixel `p`.
fn sample(p: usize, s: usize) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let offset = (s as f32 + 0.5) / SAMPLES as f32;
    #[allow(clippy::cast_precision_loss)]
    let p = p as f32;
    p + offset
}

#[cfg(test)]
#[path = "tests/badge_tests.rs"]
mod tests;
