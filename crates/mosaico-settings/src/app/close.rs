//! Asking before the window closes with unsaved edits.

use eframe::egui::{
    Align, Color32, Context, Frame, Id, Layout, Margin, Modal, RichText, Stroke, ViewportCommand,
};

use crate::theme::{self, Colors};
use crate::widgets;

/// What the user picked in the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    KeepEditing,
    Discard,
}

/// Holds a close back while there are unsaved edits, asks the user, and
/// lets the close through once they discard the edits.
#[derive(Debug, Default)]
pub struct CloseGuard {
    asking: bool,
    discarded: bool,
}

impl CloseGuard {
    /// Handles a close asked for this frame, by the window's close button
    /// (`clicked`) or by the system, such as from the taskbar, with
    /// `changes` unsaved edits.
    pub fn handle(&mut self, ctx: &Context, clicked: bool, changes: usize) {
        let from_system = ctx.input(|i| i.viewport().close_requested());
        if !clicked && !from_system {
            return;
        }
        if changes == 0 || self.discarded {
            if clicked {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            return;
        }
        if from_system {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        self.asking = true;
    }

    /// Shows the prompt while it is asking, and closes the window when
    /// the user discards the edits.
    pub fn show(&mut self, ctx: &Context, c: &Colors, changes: usize) {
        if !self.asking {
            return;
        }
        match prompt(ctx, c, changes) {
            Some(Choice::KeepEditing) => self.asking = false,
            Some(Choice::Discard) => {
                self.asking = false;
                self.discarded = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            None => {}
        }
    }
}

/// What the prompt says about `changes` edits.
fn body(changes: usize) -> String {
    match changes {
        1 => "You have 1 unsaved change. Closing the window drops it.".into(),
        n => format!("You have {n} unsaved changes. Closing the window drops them."),
    }
}

/// Draws the prompt over the window. Escape and a click beside it keep
/// editing, as the safe choice.
fn prompt(ctx: &Context, c: &Colors, changes: usize) -> Option<Choice> {
    let frame = Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, c.line))
        .corner_radius(8)
        .inner_margin(Margin::same(24));
    let modal = Modal::new(Id::unique("unsaved edits"))
        .backdrop_color(Color32::from_black_alpha(120))
        .frame(frame);
    let response = modal.show(ctx, |ui| {
        ui.set_width(380.0);
        let title = RichText::new("Discard unsaved changes?").font(theme::bold(15.0));
        ui.label(title.color(c.text));
        ui.add_space(8.0);
        let text = RichText::new(body(changes)).font(theme::regular(13.0));
        ui.label(text.color(c.dim_text));
        ui.add_space(20.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::button(ui, c, "Keep editing", true, true).clicked() {
                return Some(Choice::KeepEditing);
            }
            if widgets::button(ui, c, "Discard and close", false, true).clicked() {
                return Some(Choice::Discard);
            }
            None
        })
        .inner
    });
    response
        .inner
        .or_else(|| response.should_close().then_some(Choice::KeepEditing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_counts_the_changes_it_drops() {
        assert_eq!(
            body(1),
            "You have 1 unsaved change. Closing the window drops it."
        );
        assert_eq!(
            body(3),
            "You have 3 unsaved changes. Closing the window drops them."
        );
    }
}
