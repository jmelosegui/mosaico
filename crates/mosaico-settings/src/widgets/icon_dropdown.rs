//! A dropdown whose choices show an icon before their name.

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    ComboBox, Rect, Response, Role, TextFormat, TextStyle, Ui, WidgetInfo, style::WidgetVisuals,
    vec2,
};

use super::fields::outline;
use crate::icons::{self, Icon};
use crate::theme::Colors;

/// One choice: its value, its name and the icon drawn before the name.
pub type Choice<'a, T> = (T, &'a str, Option<Icon>);

/// Room left before a name for its icon.
const ROOM: f32 = 22.0;

/// A dropdown of `choices`, outlined in the accent when `changed`.
pub fn icon_dropdown<T: PartialEq + Clone>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    choices: &[Choice<'_, T>],
    selected: &mut T,
    changed: bool,
) -> Response {
    let current = choices.iter().find(|(value, ..)| value == selected);
    let (name, icon) = current.map_or(("", None), |(_, name, icon)| (*name, *icon));
    let dim = c.dim_text;
    let response = ui
        .scope(|ui| {
            outline(ui, c, changed);
            let inner = ComboBox::from_id_salt(label)
                .width(width)
                .selected_text(spaced(ui, c, name))
                .icon(move |ui, rect, _: &WidgetVisuals, _open| {
                    let rect = Rect::from_center_size(rect.center(), rect.size() * 1.1);
                    icons::paint(ui.painter(), rect, Icon::Chevron, dim, 1.5);
                })
                .show_ui(ui, |ui| {
                    for (value, name, icon) in choices {
                        let text = spaced(ui, c, name);
                        let item = ui.selectable_value(selected, value.clone(), text);
                        paint_icon(ui, c, &item, *icon);
                    }
                });
            paint_icon(ui, c, &inner.response, icon);
            inner.response
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, label));
    response
}

/// `name` after room for an icon.
fn spaced(ui: &Ui, c: &Colors, name: &str) -> LayoutJob {
    let format = TextFormat::simple(TextStyle::Body.resolve(ui.style()), c.text);
    let mut job = LayoutJob::default();
    job.append(name, ROOM, format);
    job
}

fn paint_icon(ui: &Ui, c: &Colors, at: &Response, icon: Option<Icon>) {
    let Some(icon) = icon else { return };
    let left = at.rect.left() + ui.spacing().button_padding.x;
    let glyph = Rect::from_min_size(
        at.rect.left_center() + vec2(left - at.rect.left(), -7.0),
        vec2(14.0, 14.0),
    );
    icons::paint(ui.painter(), glyph, icon, c.text, 1.5);
}
