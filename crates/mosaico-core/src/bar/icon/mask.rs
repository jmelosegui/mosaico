//! Turning strokes into pixels, the same on every platform.

use std::f32::consts::TAU;

use super::Stroke;

type Point = (f32, f32);

/// Points each circle and arc is drawn through.
const ARC_STEPS: usize = 32;

/// How much of each pixel of a `size` by `size` square `strokes` cover,
/// drawn to fill the square in lines `width` pixels wide: row by row,
/// from 0 for none to 255 for all of it, smoothed at the edges.
pub fn mask(strokes: &[Stroke], size: usize, width: f32) -> Vec<u8> {
    let scale = size as f32 / 24.0;
    let mut segments = Vec::new();
    for stroke in strokes {
        let scaled: Vec<Point> = points(stroke)
            .into_iter()
            .map(|(x, y)| (x * scale, y * scale))
            .collect();
        segments.extend(scaled.windows(2).map(|pair| (pair[0], pair[1])));
    }
    let reach = width / 2.0 + 0.5;
    (0..size * size)
        .map(|i| {
            let at = ((i % size) as f32 + 0.5, (i / size) as f32 + 0.5);
            let near = segments
                .iter()
                .map(|&(a, b)| distance(at, a, b))
                .fold(f32::INFINITY, f32::min);
            ((reach - near).clamp(0.0, 1.0) * 255.0).round() as u8
        })
        .collect()
}

/// The points a stroke goes through, in order.
fn points(stroke: &Stroke) -> Vec<Point> {
    match *stroke {
        Stroke::Open(points) => points.to_vec(),
        Stroke::Closed(points) => {
            let mut all = points.to_vec();
            all.extend(points.first());
            all
        }
        Stroke::Circle(center, r) => arc(center, r, 0.0, TAU),
        Stroke::Arc(center, r, from, to) => arc(center, r, from, to),
    }
}

fn arc((cx, cy): Point, r: f32, from: f32, to: f32) -> Vec<Point> {
    (0..=ARC_STEPS)
        .map(|i| from + (to - from) * i as f32 / ARC_STEPS as f32)
        .map(|a| (cx + r * a.cos(), cy + r * a.sin()))
        .collect()
}

/// How far `p` is from the segment between `a` and `b`.
fn distance(p: Point, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length).clamp(0.0, 1.0)
    };
    let (x, y) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - x).powi(2) + (p.1 - y).powi(2)).sqrt()
}
