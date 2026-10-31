//! The navigation: the search field, the pages and whether the daemon
//! runs.

use eframe::egui::{
    Align, Color32, CursorIcon, Frame, Layout, Margin, Rect, RichText, Role, Sense, TextEdit, Ui,
    WidgetInfo, vec2,
};

use super::DaemonState;
use crate::icons::{self, Icon};
use crate::pages::Page;
use crate::theme::{self, Colors};

/// Draws the navigation, switching `page` when another is clicked and
/// editing the search `query`.
pub fn show(ui: &mut Ui, c: &Colors, page: &mut Page, query: &mut String, daemon: &DaemonState) {
    search(ui, c, query);
    ui.add_space(12.0);
    ui.spacing_mut().item_spacing.y = 2.0;
    for item in Page::ALL {
        if nav_item(ui, c, item, item == *page).clicked() {
            *page = item;
        }
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| status(ui, c, daemon));
}

fn search(ui: &mut Ui, c: &Colors, query: &mut String) {
    Frame::new()
        .fill(c.control)
        .stroke(eframe::egui::Stroke::new(1.0, c.line))
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 0))
        .show(ui, |ui| {
            ui.set_height(28.0);
            ui.horizontal_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui.painter(), rect, Icon::Search, c.faint_text, 1.5);
                let hint = RichText::new("Search settings").color(c.faint_text);
                TextEdit::singleline(query)
                    .hint_text(hint)
                    .font(theme::regular(12.0))
                    .frame(eframe::egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .show(ui);
            });
        });
}

fn nav_item(ui: &mut Ui, c: &Colors, page: Page, selected: bool) -> eframe::egui::Response {
    let ready = page.is_ready();
    let sense = if ready {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), sense);
    response.widget_info(|| WidgetInfo::selected(Role::Tab, ready, selected, page.label()));
    let fill = if selected {
        c.selected
    } else if ready && response.hovered() {
        c.hover
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, fill);
    let (icon, text, font) = match (selected, ready) {
        (true, _) => (c.accent, c.text, theme::bold(13.0)),
        (false, true) => (c.dim_text, c.dim_text, theme::regular(13.0)),
        (false, false) => (c.faint_text, c.faint_text, theme::regular(13.0)),
    };
    let glyph = Rect::from_center_size(rect.left_center() + vec2(18.0, 0.0), vec2(16.0, 16.0));
    icons::paint(painter, glyph, page.icon(), icon, 1.5);
    let at = rect.left_center() + vec2(36.0, 0.0);
    painter.text(
        at,
        eframe::egui::Align2::LEFT_CENTER,
        page.label(),
        font,
        text,
    );
    if ready {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response.on_hover_text("Coming in a later release")
    }
}

fn status(ui: &mut Ui, c: &Colors, daemon: &DaemonState) {
    ui.spacing_mut().item_spacing.y = 2.0;
    let version = concat!("mosaico ", env!("CARGO_PKG_VERSION"));
    let (dot, label, detail) = match daemon {
        DaemonState::Running { version } => {
            (c.good, "Daemon running", format!("mosaico {version}"))
        }
        DaemonState::Stopped => (c.faint_text, "Daemon stopped", version.to_owned()),
    };
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(30.0);
        ui.label(
            RichText::new(detail)
                .font(theme::regular(11.0))
                .color(c.faint_text),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, dot);
        ui.add_space(2.0);
        ui.label(
            RichText::new(label)
                .font(theme::regular(12.0))
                .color(c.text),
        );
    });
}
