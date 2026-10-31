//! The bar under the page: how many edits there are, what became of the
//! last Apply, and the buttons that revert or apply the edits.

use eframe::egui::{Align, Color32, Layout, RichText, Sense, Ui, vec2};

use super::Applied;
use crate::theme::{self, Colors};
use crate::widgets;

/// What the footer shows.
pub struct State<'a> {
    /// How many settings were edited.
    pub changes: usize,
    /// What became of the last Apply.
    pub applied: Option<&'a Applied>,
    /// Whether the settings cannot be saved, because the file has a
    /// problem.
    pub blocked: bool,
}

/// What the user clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Apply,
    Revert,
}

/// What the footer says about `changes` edits.
pub fn summary(changes: usize) -> String {
    match changes {
        0 => "No unsaved changes".into(),
        1 => "1 unsaved change".into(),
        n => format!("{n} unsaved changes"),
    }
}

/// What the footer says, and in which color: a failed Apply until the
/// next one, that the edits were saved until the next edit, or else how
/// many edits there are.
fn status(c: &Colors, state: &State) -> (String, Color32) {
    match state.applied {
        Some(Applied::Failed(why)) => (format!("Could not save: {why}"), c.bad),
        Some(Applied::Saved) if state.changes == 0 => ("Changes saved".into(), c.good),
        _ => (summary(state.changes), c.dim_text),
    }
}

/// Draws the footer, returning the button clicked, if any.
pub fn show(ui: &mut Ui, c: &Colors, state: &State) -> Option<Action> {
    let mut action = None;
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        if state.changes > 0 {
            let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, c.accent);
        }
        let (text, color) = status(c, state);
        ui.label(RichText::new(text).font(theme::regular(12.0)).color(color));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let can_apply = state.changes > 0 && !state.blocked;
            let apply = widgets::button(ui, c, "Apply", true, can_apply);
            let apply = if state.blocked {
                apply.on_disabled_hover_text("Fix the file to save here.")
            } else {
                apply
            };
            if apply.clicked() {
                action = Some(Action::Apply);
            }
            if widgets::button(ui, c, "Revert", false, state.changes > 0).clicked() {
                action = Some(Action::Revert);
            }
        });
    });
    action
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
