//! Line icons, drawn on a 24 unit grid and scaled to their rect.

use std::f32::consts::PI;

use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2};

/// An icon the window draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    General,
    Layout,
    Workspaces,
    Borders,
    Mouse,
    Animations,
    Bar,
    Keybindings,
    Rules,
    Search,
    Chevron,
    Reset,
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// One stroke of an icon, in grid units.
enum Part {
    Open(&'static [(f32, f32)]),
    Closed(&'static [(f32, f32)]),
    Circle((f32, f32), f32),
    /// An arc around a center, from one angle to another, in radians.
    Arc((f32, f32), f32, f32, f32),
}

use Part::{Arc, Circle, Closed, Open};

const fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> [(f32, f32); 4] {
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

const FRAME: [(f32, f32); 4] = rect(3.0, 3.0, 21.0, 21.0);

fn parts(icon: Icon) -> &'static [Part] {
    match icon {
        Icon::General => &[
            Open(&[(3.0, 4.0), (10.0, 4.0)]),
            Open(&[(14.0, 4.0), (21.0, 4.0)]),
            Open(&[(3.0, 12.0), (8.0, 12.0)]),
            Open(&[(12.0, 12.0), (21.0, 12.0)]),
            Open(&[(3.0, 20.0), (12.0, 20.0)]),
            Open(&[(16.0, 20.0), (21.0, 20.0)]),
            Open(&[(14.0, 2.0), (14.0, 6.0)]),
            Open(&[(8.0, 10.0), (8.0, 14.0)]),
            Open(&[(16.0, 18.0), (16.0, 22.0)]),
        ],
        Icon::Layout => &[
            Closed(&FRAME),
            Open(&[(12.0, 3.0), (12.0, 21.0)]),
            Open(&[(12.0, 12.0), (21.0, 12.0)]),
        ],
        Icon::Workspaces => &[
            Closed(&[(12.0, 2.0), (2.0, 7.0), (12.0, 12.0), (22.0, 7.0)]),
            Open(&[(2.0, 12.0), (12.0, 17.0), (22.0, 12.0)]),
            Open(&[(2.0, 17.0), (12.0, 22.0), (22.0, 17.0)]),
        ],
        Icon::Borders => &[
            Closed(&FRAME),
            Closed(&const { rect(7.0, 7.0, 17.0, 17.0) }),
        ],
        Icon::Mouse => &[Closed(&[
            (5.0, 3.0),
            (19.0, 10.0),
            (13.0, 12.0),
            (11.0, 18.0),
        ])],
        Icon::Animations => &[Closed(&[
            (13.0, 2.0),
            (3.0, 14.0),
            (12.0, 14.0),
            (11.0, 22.0),
            (21.0, 10.0),
            (12.0, 10.0),
        ])],
        Icon::Bar => &[Closed(&FRAME), Open(&[(3.0, 9.0), (21.0, 9.0)])],
        Icon::Keybindings => &[
            Closed(&const { rect(2.0, 6.0, 22.0, 18.0) }),
            Open(&[(6.0, 10.0), (8.0, 10.0)]),
            Open(&[(11.0, 10.0), (13.0, 10.0)]),
            Open(&[(16.0, 10.0), (18.0, 10.0)]),
            Open(&[(7.0, 14.0), (17.0, 14.0)]),
        ],
        Icon::Rules => &[
            Open(&[(3.0, 6.0), (5.0, 8.0), (9.0, 4.0)]),
            Open(&[(3.0, 14.0), (5.0, 16.0), (9.0, 12.0)]),
            Open(&[(13.0, 6.0), (21.0, 6.0)]),
            Open(&[(13.0, 12.0), (21.0, 12.0)]),
            Open(&[(13.0, 18.0), (21.0, 18.0)]),
        ],
        Icon::Search => &[
            Circle((11.0, 11.0), 7.0),
            Open(&[(16.0, 16.0), (21.0, 21.0)]),
        ],
        Icon::Chevron => &[Open(&[(6.0, 9.0), (12.0, 15.0), (18.0, 9.0)])],
        Icon::Reset => &[
            Arc((12.0, 12.0), 9.0, PI, -2.3),
            Open(&[(3.0, 3.0), (3.0, 8.0), (8.0, 8.0)]),
        ],
        Icon::Minimize => &[Open(&[(5.0, 12.0), (19.0, 12.0)])],
        Icon::Maximize => &[Closed(&const { rect(5.0, 5.0, 19.0, 19.0) })],
        Icon::Restore => &[
            Closed(&const { rect(5.0, 8.0, 16.0, 19.0) }),
            Open(&[
                (8.0, 8.0),
                (8.0, 5.0),
                (19.0, 5.0),
                (19.0, 16.0),
                (16.0, 16.0),
            ]),
        ],
        Icon::Close => &[
            Open(&[(6.0, 6.0), (18.0, 18.0)]),
            Open(&[(18.0, 6.0), (6.0, 18.0)]),
        ],
    }
}

/// Draws `icon` filling `rect`, with lines `width` points wide.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32, width: f32) {
    let scale = rect.width().min(rect.height()) / 24.0;
    let origin = rect.center() - eframe::egui::vec2(12.0, 12.0) * scale;
    let at = |(x, y): (f32, f32)| origin + eframe::egui::vec2(x, y) * scale;
    let stroke = Stroke::new(width, color);
    for part in parts(icon) {
        let shape = match part {
            Open(points) => Shape::line(points.iter().copied().map(at).collect(), stroke),
            Closed(points) => Shape::closed_line(points.iter().copied().map(at).collect(), stroke),
            Circle(center, r) => Shape::circle_stroke(at(*center), r * scale, stroke),
            Arc(center, r, from, to) => {
                Shape::line(arc(at(*center), r * scale, *from, *to), stroke)
            }
        };
        painter.add(shape);
    }
}

fn arc(center: Pos2, r: f32, from: f32, to: f32) -> Vec<Pos2> {
    const STEPS: usize = 32;
    (0..=STEPS)
        .map(|i| from + (to - from) * i as f32 / STEPS as f32)
        .map(|a| pos2(center.x + r * a.cos(), center.y + r * a.sin()))
        .collect()
}
