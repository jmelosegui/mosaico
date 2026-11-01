//! The picture of an easing curve and the points that make it.

use eframe::egui::{Frame, Id, Margin, Pos2, RichText, Sense, Shape, Stroke, Ui, pos2, vec2};
use mosaico_core::motion::Easing;

use crate::theme::{self, Colors};
use crate::widgets;

use super::play::play_button;

/// The size of the picture.
const BOX: (f32, f32) = (232.0, 104.0);

/// Space around the curve in the picture.
const INSET: f32 = 16.0;

/// The picture of `shape` with the four `points` under the easing row,
/// editable when `custom`. Play runs a dot along the curve in `seconds`,
/// as long as a transition takes. Returns whether a point was edited.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    shape: Option<Easing>,
    points: &mut [f64; 4],
    custom: bool,
    seconds: f64,
) -> bool {
    let mut edited = false;
    Frame::new()
        .inner_margin(Margin {
            left: 16,
            right: 16,
            top: 0,
            bottom: 16,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                let play = ui.make_persistent_id("easing-play");
                preview(ui, c, shape, play, seconds);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    ui.label(
                        RichText::new("Custom curve")
                            .font(theme::bold(12.0))
                            .color(c.dim_text),
                    );
                    let help =
                        "Choose Custom to enter cubic-bezier(x1, y1, x2, y2), each from 0 to 1.";
                    ui.label(
                        RichText::new(help)
                            .font(theme::regular(11.0))
                            .color(c.dim_text),
                    );
                    edited = fields(ui, c, points, custom);
                    if play_button(ui, c).clicked() {
                        let now = ui.input(|i| i.time);
                        ui.data_mut(|d| d.insert_temp(play, now));
                        ui.request_repaint();
                    }
                });
            });
        });
    edited
}

fn fields(ui: &mut Ui, c: &Colors, points: &mut [f64; 4], enabled: bool) -> bool {
    const NAMES: [&str; 4] = ["Curve x1", "Curve y1", "Curve x2", "Curve y2"];
    let mut edited = false;
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (point, name) in points.iter_mut().zip(NAMES) {
                let field =
                    widgets::number_of_width(ui, c, name, 56.0, point, 0.0..=1.0, 0.01, "", false);
                edited |= field.changed();
                ui.add_space(6.0);
            }
        });
    });
    edited
}

/// The curve over its straight line, with a dot running along it for
/// `seconds` after Play was clicked.
fn preview(ui: &mut Ui, c: &Colors, shape: Option<Easing>, play: Id, seconds: f64) {
    let (rect, _) = ui.allocate_exact_size(vec2(BOX.0, BOX.1), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, c.control);
    let area = rect.shrink(INSET);
    let at = |t: f64, progress: f64| -> Pos2 {
        pos2(
            area.left() + area.width() * t as f32,
            area.bottom() - area.height() * progress as f32,
        )
    };
    painter.line_segment([at(0.0, 0.0), at(1.0, 1.0)], Stroke::new(1.0, c.raised));
    let Some(easing) = shape else { return };
    const STEPS: u32 = 32;
    let curve: Vec<Pos2> = (0..=STEPS)
        .map(|i| {
            let t = f64::from(i) / f64::from(STEPS);
            at(t, easing.progress(t))
        })
        .collect();
    painter.add(Shape::line(curve, Stroke::new(2.0, c.accent)));
    painter.circle_filled(at(0.0, 0.0), 3.0, c.accent);
    painter.circle_filled(at(1.0, 1.0), 3.0, c.accent);
    let Some(started) = ui.data(|d| d.get_temp::<f64>(play)) else {
        return;
    };
    let t = (ui.input(|i| i.time) - started) / seconds;
    if seconds <= 0.0 || t >= 1.0 {
        ui.data_mut(|d| d.remove_temp::<f64>(play));
        return;
    }
    ui.painter()
        .circle_filled(at(t, easing.progress(t)), 5.0, c.text);
    ui.request_repaint();
}
