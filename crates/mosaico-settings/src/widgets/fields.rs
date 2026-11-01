//! A drop-down list, a number field and a text field, in the shape of
//! the other controls.

use eframe::egui::{
    ComboBox, DragValue, Frame, Margin, Response, Role, Stroke, TextEdit, Ui, Widget, WidgetInfo,
    emath::Numeric, style::WidgetVisuals,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// Outlines the controls of a changed setting in the accent color.
fn outline(ui: &mut Ui, c: &Colors, changed: bool) {
    if changed {
        let w = &mut ui.visuals_mut().widgets;
        for state in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.bg_stroke = Stroke::new(1.0, c.accent);
        }
    }
}

/// A drop-down `width` points wide that picks one of `options`, shown
/// by their labels. `label` names it for assistive technology.
pub fn dropdown<T: PartialEq + Clone>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    options: &[(T, &str)],
    selected: &mut T,
    changed: bool,
) -> Response {
    let current = options
        .iter()
        .find(|(value, _)| value == selected)
        .map_or("", |(_, label)| label);
    let response = ui
        .scope(|ui| {
            outline(ui, c, changed);
            let dim = c.dim_text;
            ComboBox::from_id_salt(label)
                .width(width)
                .selected_text(current)
                .icon(move |ui, rect, _: &WidgetVisuals, _open| {
                    let rect =
                        eframe::egui::Rect::from_center_size(rect.center(), rect.size() * 1.1);
                    icons::paint(ui.painter(), rect, Icon::Chevron, dim, 1.5);
                })
                .show_ui(ui, |ui| {
                    for (value, label) in options {
                        ui.selectable_value(selected, value.clone(), *label);
                    }
                })
                .response
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, label));
    response
}

/// A number in `range` followed by its `unit`, which can be typed or
/// dragged, `speed` per point dragged.
pub fn number<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    let stroke = if changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    Frame::new()
        .fill(c.control)
        .stroke(stroke)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(4, 0))
        .show(ui, |ui| {
            ui.set_height(26.0);
            let w = &mut ui.visuals_mut().widgets;
            for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                state.weak_bg_fill = eframe::egui::Color32::TRANSPARENT;
                state.bg_fill = c.control;
            }
            ui.horizontal_centered(|ui| {
                let field = ui
                    .add_sized(
                        [56.0, 22.0],
                        DragValue::new(value).range(range).speed(speed),
                    )
                    .on_hover_text("Drag, or click to type");
                let unit = eframe::egui::RichText::new(unit).font(theme::regular(12.0));
                ui.label(unit.color(c.faint_text));
                ui.add_space(4.0);
                field
            })
            .inner
        })
        .inner
}

/// A line of text `width` points wide, showing `hint` while it is empty.
/// `label` names it for assistive technology.
pub fn text(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    value: &mut String,
    hint: &str,
    changed: bool,
) -> Response {
    let stroke = if changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    let response = Frame::new()
        .fill(c.control)
        .stroke(stroke)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(6, 0))
        .show(ui, |ui| {
            ui.set_height(26.0);
            ui.horizontal_centered(|ui| {
                let field = TextEdit::singleline(value)
                    .frame(Frame::NONE)
                    .hint_text(hint)
                    .font(theme::regular(12.0))
                    .desired_width(width);
                ui.add(field)
            })
            .inner
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, label));
    response
}

/// A button, filled with the accent when it is the `primary` one.
pub fn button(ui: &mut Ui, c: &Colors, label: &str, primary: bool, enabled: bool) -> Response {
    let (fill, text) = if primary {
        (c.accent, c.on_accent)
    } else {
        (c.raised, c.text)
    };
    let text = eframe::egui::RichText::new(label)
        .font(theme::bold(13.0))
        .color(text);
    let button = eframe::egui::Button::new(text)
        .fill(fill)
        .corner_radius(5)
        .min_size(eframe::egui::vec2(0.0, 30.0));
    ui.add_enabled_ui(enabled, |ui| {
        ui.spacing_mut().button_padding = eframe::egui::vec2(16.0, 6.0);
        button.ui(ui)
    })
    .inner
}
