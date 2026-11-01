//! The Mouse tab: what the bar takes of the pointer and what a click on
//! a widget does.

use eframe::egui::{Align, CornerRadius, Frame, Layout, Margin, RichText, Ui, vec2};
use mosaico_core::bar::WidgetClicks;
use mosaico_core::config::{ClickBinding, WidgetKind};

use super::BarPage;
use super::columns::label as widget_label;
use crate::icons::Icon;
use crate::pages::keybindings::labels;
use crate::theme::{self, Colors};
use crate::widgets;

const BUTTONS: [(&str, &str); 3] = [("left", "Left"), ("right", "Right"), ("middle", "Middle")];

/// How wide the drop-downs of a click are: the widget, the button and
/// the action.
const WIDTHS: [f32; 3] = [170.0, 120.0, 200.0];

const HELP: &str = "A click on a workspace number always goes to that workspace. Workspace and layout actions act on the bar's own monitor.";

impl BarPage {
    pub(super) fn mouse(&mut self, ui: &mut Ui, c: &Colors) {
        self.sections(
            ui,
            c,
            "",
            &[("Pointer", &["mouse.enabled", "mouse.scroll"])],
        );
        self.click_table(ui, c);
    }

    pub(super) fn click_table(&mut self, ui: &mut Ui, c: &Colors) {
        widgets::heading(ui, c, "Click actions", |_| {});
        ui.label(
            RichText::new(HELP)
                .font(theme::regular(11.0))
                .color(c.dim_text),
        );
        ui.add_space(6.0);
        let mut removed = None;
        let saved = self.saved.clicks.clone();
        widgets::card(ui, c, |ui| {
            header(ui, c);
            for (i, click) in self.edit.clicks.iter_mut().enumerate() {
                widgets::divider(ui, c);
                let unsaved = !saved.contains(click);
                if row(ui, c, i, click, unsaved) {
                    removed = Some(i);
                }
            }
        });
        if let Some(i) = removed {
            self.edit.clicks.remove(i);
        }
        ui.add_space(6.0);
        if widgets::ghost_button(ui, c, "Add click action").clicked() {
            self.edit.clicks.push(new_click(&self.edit.clicks));
        }
        ui.add_space(16.0);
    }
}

/// A binding on a button no click of the workspaces has yet.
fn new_click(clicks: &[ClickBinding]) -> ClickBinding {
    let taken = |button: &str| {
        clicks.iter().any(|c| {
            c.widget.as_deref() == Some("workspaces") && c.button.as_deref() == Some(button)
        })
    };
    let button = BUTTONS.iter().map(|(b, _)| *b).find(|b| !taken(b));
    ClickBinding::new("workspaces", button.unwrap_or("middle"), "retile")
}

fn header(ui: &mut Ui, c: &Colors) {
    Frame::new()
        .fill(c.control)
        .corner_radius(CornerRadius {
            nw: 8,
            ne: 8,
            sw: 0,
            se: 0,
        })
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                for (title, width) in ["Widget", "Button", "Action"].iter().zip(WIDTHS) {
                    let size = vec2(width, 16.0);
                    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                        ui.set_min_width(width);
                        let text = RichText::new(*title).font(theme::bold(11.0));
                        ui.label(text.color(c.dim_text));
                    });
                }
            });
        });
}

/// One click's row. Returns whether it should be removed.
fn row(ui: &mut Ui, c: &Colors, index: usize, click: &mut ClickBinding, unsaved: bool) -> bool {
    let n = index + 1;
    let mut remove = false;
    Frame::new()
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                let widget_options: Vec<_> = WidgetKind::ALL
                    .iter()
                    .map(|k| (k.name().to_owned(), widget_label(*k)))
                    .collect();
                choose(
                    ui,
                    c,
                    &format!("Widget of click {n}"),
                    WIDTHS[0],
                    &widget_options,
                    &mut click.widget,
                    unsaved,
                );
                let buttons: Vec<_> = BUTTONS.iter().map(|(b, l)| ((*b).to_owned(), *l)).collect();
                choose(
                    ui,
                    c,
                    &format!("Button of click {n}"),
                    WIDTHS[1],
                    &buttons,
                    &mut click.button,
                    unsaved,
                );
                let names: Vec<(String, String)> = WidgetClicks::actions()
                    .into_iter()
                    .map(|a| (a.to_string(), labels::label(a)))
                    .collect();
                let actions: Vec<_> = names.iter().map(|(a, l)| (a.clone(), l.as_str())).collect();
                choose(
                    ui,
                    c,
                    &format!("Action of click {n}"),
                    WIDTHS[2],
                    &actions,
                    &mut click.action,
                    unsaved,
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let label = format!("Remove click action {n}");
                    remove = widgets::icon_button(ui, c, Icon::Trash, &label, c.dim_text).clicked();
                });
            });
        });
    remove
}

/// A drop-down over `options` for a key of a click, which also offers
/// the value the file has when it is none of them.
fn choose(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    options: &[(String, &str)],
    value: &mut Option<String>,
    changed: bool,
) {
    let current = value.clone().unwrap_or_default();
    let mut chosen = current.clone();
    let mut all = options.to_vec();
    if !all.iter().any(|(name, _)| *name == current) {
        all.push((current.clone(), current.as_str()));
    }
    widgets::dropdown(ui, c, label, width, &all, &mut chosen, changed);
    if chosen != current {
        *value = Some(chosen);
    }
}
