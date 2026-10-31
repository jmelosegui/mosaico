//! Tabs across the top of a page, underlined under the selected one.

use eframe::egui::{CursorIcon, Response, Role, Sense, Ui, WidgetInfo, pos2, vec2};

use crate::theme::{self, Colors};

/// One tab for each of `names`, over a line. Clicking one sets
/// `selected` to its index.
pub fn tabs(ui: &mut Ui, c: &Colors, names: &[&str], selected: &mut usize) -> Response {
    let top = ui.cursor().min;
    let width = ui.available_width();
    let response = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 20.0;
            for (i, name) in names.iter().enumerate() {
                if tab(ui, c, name, i == *selected).clicked() {
                    *selected = i;
                }
            }
        })
        .response;
    let y = response.rect.bottom();
    let line = [pos2(top.x, y), pos2(top.x + width, y)];
    ui.painter()
        .line_segment(line, eframe::egui::Stroke::new(1.0, c.line));
    ui.add_space(12.0);
    response
}

fn tab(ui: &mut Ui, c: &Colors, name: &str, on: bool) -> Response {
    let font = if on {
        theme::bold(13.0)
    } else {
        theme::regular(13.0)
    };
    let color = if on { c.text } else { c.dim_text };
    let galley = ui.painter().layout_no_wrap(name.to_owned(), font, color);
    let size = vec2(galley.size().x, galley.size().y + 9.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Tab, true, on, name));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let color = if response.hovered() && !on {
            c.text
        } else {
            color
        };
        painter.galley_with_override_text_color(rect.min, galley, color);
        if on {
            let underline = eframe::egui::Rect::from_min_max(
                pos2(rect.left(), rect.bottom() - 2.0),
                rect.right_bottom(),
            );
            painter.rect_filled(underline, 1.0, c.accent);
        }
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}
