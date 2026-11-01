//! The widget order editor: the widgets on each side of the bar, with
//! buttons to move them and a switch to enable them.

use eframe::egui::{
    Align, Frame, Layout, Margin, Painter, Pos2, Rect, RichText, Role, Sense, Stroke, Ui, Vec2,
    WidgetInfo, vec2,
};
use mosaico_core::config::{BarWidget, BarWidgets, BarZone, WidgetKind};

use crate::pages::Row;
use crate::theme::{self, Colors};
use crate::widgets;

/// What a click on one of a widget's buttons asks for.
enum Move {
    /// One place nearer the start of its side.
    Up,
    /// One place nearer the end of its side.
    Down,
    /// To the end of the other side.
    Across,
}

/// A widget of the editor, and what to do with it.
struct Action {
    zone: BarZone,
    index: usize,
    change: Move,
}

/// How the widget is called in the window.
fn label(kind: WidgetKind) -> &'static str {
    match kind {
        WidgetKind::Workspaces => "Workspaces",
        WidgetKind::Layout => "Layout",
        WidgetKind::Clock => "Clock",
        WidgetKind::Date => "Date",
        WidgetKind::Ram => "Memory",
        WidgetKind::Cpu => "Processor",
        WidgetKind::Update => "Update notice",
        WidgetKind::ActiveWindow => "Active window",
        WidgetKind::Media => "Media",
        WidgetKind::Paused => "Paused mark",
    }
}

/// Draws the editor when it matches `query`, changing `edit`. Returns
/// whether it was drawn.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    query: &str,
    saved: &BarWidgets,
    edit: &mut BarWidgets,
) -> bool {
    let help = "Widgets on the left are drawn from the left edge, those on the right from the right edge. Move one nearer the middle or to the other side, or switch it off.";
    if !Row::new("Widgets", help).matches(query) {
        return false;
    }
    ui.label(
        RichText::new(help)
            .font(theme::regular(11.0))
            .color(c.dim_text),
    );
    ui.add_space(8.0);
    let mut action = None;
    for zone in BarZone::ALL {
        let title = match zone {
            BarZone::Left => "Left widgets",
            BarZone::Right => "Right widgets",
        };
        widgets::section(ui, c, title, |ui| {
            let list = edit.zone_mut(zone);
            if list.is_empty() {
                empty(ui, c);
            }
            let count = list.len();
            for (index, widget) in list.iter_mut().enumerate() {
                if index > 0 {
                    widgets::divider(ui, c);
                }
                let unsaved = saved.zone(zone).get(index) != Some(widget);
                let at = Place { zone, index, count };
                if let Some(change) = entry(ui, c, widget, &at, unsaved) {
                    action = Some(Action {
                        zone,
                        index,
                        change,
                    });
                }
            }
        });
    }
    if let Some(Action {
        zone,
        index,
        change,
    }) = action
    {
        match change {
            Move::Up => edit.move_to(zone, index, zone, index.saturating_sub(1)),
            Move::Down => edit.move_to(zone, index, zone, index + 1),
            Move::Across => edit.move_to(zone, index, zone.other(), usize::MAX),
        }
    }
    true
}

/// Where a widget is.
struct Place {
    zone: BarZone,
    index: usize,
    count: usize,
}

fn empty(ui: &mut Ui, c: &Colors) {
    Frame::new()
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.label(
                RichText::new("No widgets on this side.")
                    .font(theme::regular(12.0))
                    .color(c.dim_text),
            );
        });
}

/// One widget's row.
fn entry(
    ui: &mut Ui,
    c: &Colors,
    widget: &mut BarWidget,
    at: &Place,
    unsaved: bool,
) -> Option<Move> {
    let name = label(widget.kind);
    let mut change = None;
    Frame::new()
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            let size = vec2(ui.available_width(), 28.0);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                widgets::toggle(ui, c, name, &mut widget.enabled);
                ui.add_space(8.0);
                let (across, towards) = match at.zone {
                    BarZone::Left => ("right", Vec2::X),
                    BarZone::Right => ("left", -Vec2::X),
                };
                let label = format!("Move {name} to the {across} side");
                if icon_button(ui, c, towards, &label, true) {
                    change = Some(Move::Across);
                }
                let label = format!("Move {name} down");
                if icon_button(ui, c, Vec2::Y, &label, at.index + 1 < at.count) {
                    change = Some(Move::Down);
                }
                let label = format!("Move {name} up");
                if icon_button(ui, c, -Vec2::Y, &label, at.index > 0) {
                    change = Some(Move::Up);
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let color = if widget.enabled { c.text } else { c.faint_text };
                    ui.label(RichText::new(name).font(theme::bold(13.0)).color(color));
                    if unsaved {
                        let (rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 3.0, c.accent);
                    }
                });
            });
        });
    change
}

/// A square button with an arrow pointing `towards`, read out as
/// `label`. Returns whether it was clicked.
fn icon_button(ui: &mut Ui, c: &Colors, towards: Vec2, label: &str, enabled: bool) -> bool {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 26.0), sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
    let fill = if enabled && response.hovered() {
        c.hover
    } else {
        c.control
    };
    let color = if enabled { c.text } else { c.line };
    ui.painter().rect_filled(rect, 5.0, fill);
    arrow(ui.painter(), rect, towards, color);
    enabled && response.clicked()
}

fn arrow(painter: &Painter, rect: Rect, towards: Vec2, color: eframe::egui::Color32) {
    let stroke = Stroke::new(1.5, color);
    let side = vec2(-towards.y, towards.x);
    let tip: Pos2 = rect.center() + towards * 5.0;
    painter.line_segment([rect.center() - towards * 5.0, tip], stroke);
    for wing in [side, -side] {
        painter.line_segment([tip, tip - towards * 4.0 + wing * 4.0], stroke);
    }
}
