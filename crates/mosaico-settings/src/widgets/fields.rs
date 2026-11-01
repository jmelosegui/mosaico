//! A drop-down list, a number field and a text field, in the shape of
//! the other controls.

use eframe::egui::{
    Align, Color32, ComboBox, DragValue, Frame, Layout, Margin, Response, RichText, Role, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Widget, WidgetInfo, emath::Numeric,
    style::WidgetVisuals, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// Outlines the controls of a changed setting in the accent color.
pub(super) fn outline(ui: &mut Ui, c: &Colors, changed: bool) {
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
    dimmed_dropdown(ui, c, label, width, options, selected, (changed, false))
}

/// A [`dropdown`] whose selection reads dim when `dimmed`, for a choice
/// that stands for a default. `state` is whether it is `changed` and
/// `dimmed`.
pub fn dimmed_dropdown<T: PartialEq + Clone>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    options: &[(T, &str)],
    selected: &mut T,
    (changed, dimmed): (bool, bool),
) -> Response {
    let current = options
        .iter()
        .find(|(value, _)| value == selected)
        .map_or("", |(_, label)| label);
    let current = if dimmed {
        RichText::new(current).color(c.faint_text)
    } else {
        RichText::new(current)
    };
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

/// A number in `range` and its `unit` after it, which can be typed or
/// dragged, `speed` per point dragged. `label` names it for assistive
/// technology.
#[allow(clippy::too_many_arguments)]
pub fn number<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    number_of_width(
        ui,
        c,
        label,
        NUMBER_WIDTH,
        value,
        range,
        speed,
        unit,
        changed,
    )
}

/// A [`number`] `width` points wide, for rows that line several up.
#[allow(clippy::too_many_arguments)]
pub fn number_of_width<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let stroke = if changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    ui.painter()
        .rect(rect, 5.0, c.control, stroke, StrokeKind::Inside);
    let inner =
        eframe::egui::Rect::from_min_max(rect.min + vec2(10.0, 2.0), rect.max - vec2(8.0, 2.0));
    let builder = UiBuilder::new()
        .max_rect(inner)
        .layout(Layout::left_to_right(Align::Center));
    // A child the parent does not lay out, so its cursor stays after the
    // whole field.
    let mut child = ui.new_child(builder);
    let ui = &mut child;
    {
        let w = &mut ui.visuals_mut().widgets;
        for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
            state.weak_bg_fill = Color32::TRANSPARENT;
            state.bg_fill = c.control;
        }
        // The number sits on the left, as narrow as its text, and the unit
        // on the right.
        ui.spacing_mut().interact_size = vec2(0.0, 22.0);
        ui.spacing_mut().button_padding.x = 0.0;
        let field = ui
            .add(drag_value(value, range, speed))
            .on_hover_text("Drag, or click to type");
        field.widget_info(|| WidgetInfo::labeled(Role::SpinButton, true, label));
        if !unit.is_empty() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let unit = RichText::new(unit).font(theme::regular(12.0));
                ui.label(unit.color(c.faint_text));
            });
        }
        let blank = eframe::egui::Rect::from_min_max(
            eframe::egui::pos2(field.rect.right(), inner.top()),
            inner.right_bottom(),
        );
        let blank = ui.interact(blank, field.id.with("blank"), Sense::click());
        if blank
            .on_hover_cursor(eframe::egui::CursorIcon::Text)
            .clicked()
        {
            ui.memory_mut(|m| m.request_focus(field.id));
        }
        field
    }
}

/// A field for `value`, showing two decimals when it has any.
pub(super) fn drag_value<N: Numeric>(
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
) -> DragValue<'_> {
    let field = DragValue::new(value).range(range).speed(speed);
    if N::INTEGRAL {
        field
    } else {
        field.fixed_decimals(2)
    }
}

/// How wide a [`number`] is.
pub(super) const NUMBER_WIDTH: f32 = 96.0;

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
                    .align(eframe::egui::Align2::LEFT_CENTER)
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
