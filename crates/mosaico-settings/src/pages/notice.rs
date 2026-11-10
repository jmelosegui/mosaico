//! What the page says about its file instead of, or over, its rows.

use eframe::egui::{Frame, Margin, RichText, Stroke, Ui};

use crate::theme::{self, Colors};

/// Why `file` cannot be edited: what is wrong with it, over the page.
pub fn problem(ui: &mut Ui, c: &Colors, file: &str, problem: &str) {
    Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, c.warning))
        .corner_radius(8)
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let title = format!("{file} has a problem, so it can't be edited here");
            ui.label(RichText::new(title).font(theme::bold(13.0)));
            ui.add_space(4.0);
            ui.label(
                RichText::new(problem)
                    .font(theme::regular(12.0))
                    .color(c.dim_text),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Fix it in an editor; this window picks the fix up.")
                    .font(theme::regular(11.0))
                    .color(c.faint_text),
            );
        });
    ui.add_space(16.0);
}

/// Why the rows of `file` are not shown: mosaico does not read it with
/// the bar picked.
pub fn not_read(ui: &mut Ui, c: &Colors, file: &str) {
    let text = format!(
        "mosaico does not read {file} while another bar is picked. Pick the mosaico bar to \
         style it here."
    );
    ui.label(
        RichText::new(text)
            .font(theme::regular(12.0))
            .color(c.dim_text),
    );
    ui.add_space(16.0);
}
