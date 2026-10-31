//! The bar under the page: how many edits there are, and the buttons
//! that revert or apply them.

use eframe::egui::{Align, Layout, RichText, Sense, Ui, vec2};

use crate::model::{Draft, General};
use crate::theme::{self, Colors};
use crate::widgets;

/// What the footer says about `changes` edits.
pub fn summary(changes: usize) -> String {
    match changes {
        0 => "No unsaved changes".into(),
        1 => "1 unsaved change".into(),
        n => format!("{n} unsaved changes"),
    }
}

/// Draws the footer for `draft`, reverting it when Revert is clicked.
pub fn show(ui: &mut Ui, c: &Colors, draft: &mut Draft<General>) {
    let changes = draft.changes();
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        if changes > 0 {
            let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, c.accent);
        }
        let text = RichText::new(summary(changes))
            .font(theme::regular(12.0))
            .color(c.dim_text);
        ui.label(text);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            widgets::button(ui, c, "Apply", true, false)
                .on_disabled_hover_text("Saving arrives in a later release.");
            if widgets::button(ui, c, "Revert", false, changes > 0).clicked() {
                draft.revert();
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_the_changes_in_words() {
        assert_eq!(summary(0), "No unsaved changes");
        assert_eq!(summary(1), "1 unsaved change");
        assert_eq!(summary(3), "3 unsaved changes");
    }
}
