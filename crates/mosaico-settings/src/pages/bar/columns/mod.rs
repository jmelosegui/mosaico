//! The Widgets tab: the widgets on each side of the bar, which can be
//! dragged to reorder, switched off and opened for their options.

use eframe::egui::{Frame, Id, Popup, Rect, RichText, Stroke, Ui, pos2};
use mosaico_core::config::{BarWidget, BarWidgets, BarZone, Theme, WidgetKind};

use super::BarPage;
use crate::icons::Icon;
use crate::theme::{self, Colors};
use crate::widgets;

mod glyphs;
mod row;
use row::row;

/// How tall a widget's row is.
const ROW: f32 = 40.0;

/// How a widget is called in the window.
pub fn label(kind: WidgetKind) -> &'static str {
    match kind {
        WidgetKind::Workspaces => "Workspaces",
        WidgetKind::Layout => "Layout",
        WidgetKind::Clock => "Clock",
        WidgetKind::Date => "Date",
        WidgetKind::Ram => "RAM",
        WidgetKind::Cpu => "CPU",
        WidgetKind::Update => "Update",
        WidgetKind::ActiveWindow => "Active window",
        WidgetKind::Media => "Media",
        WidgetKind::Paused => "Paused",
    }
}

/// The icon beside a widget's name.
fn icon(kind: WidgetKind) -> Icon {
    match kind {
        WidgetKind::Workspaces => Icon::Grid,
        WidgetKind::Layout => Icon::Layout,
        WidgetKind::Clock => Icon::Clock,
        WidgetKind::Date => Icon::Calendar,
        WidgetKind::Ram => Icon::Memory,
        WidgetKind::Cpu => Icon::Processor,
        WidgetKind::Update => Icon::Download,
        WidgetKind::ActiveWindow => Icon::Window,
        WidgetKind::Media => Icon::Music,
        WidgetKind::Paused => Icon::Pause,
    }
}

/// What is said under a widget's name, when something is.
fn note(widget: &BarWidget) -> Option<&str> {
    match widget.kind {
        WidgetKind::Date => widget.format.as_deref(),
        WidgetKind::Update => Some("Only when an update is available"),
        WidgetKind::Paused => Some("Only while paused"),
        WidgetKind::Media => Some("Only while something plays"),
        _ => None,
    }
}

/// What the tab remembers between frames.
#[derive(Debug, Default)]
pub struct Editor {
    /// The widget whose options are shown.
    open: Option<(BarZone, usize)>,
    /// The widget being dragged by its grip.
    drag: Option<(BarZone, usize)>,
    /// The grip that takes the keyboard focus on the next frame, after
    /// a move that went with the keys.
    focus: Option<Id>,
}

/// A change the frame asks for, made once every row is drawn.
enum Change {
    Move {
        from: (BarZone, usize),
        to: (BarZone, usize),
        by_key: bool,
    },
    Add(BarZone, WidgetKind),
}

/// Where each row and column was drawn this frame.
#[derive(Default)]
struct Seen {
    rows: Vec<(BarZone, usize, Rect)>,
    columns: Vec<(BarZone, Rect)>,
    released: bool,
}

/// What drawing a column reads and edits.
struct Ctx<'a> {
    c: &'a Colors,
    theme: Theme,
    saved: &'a BarWidgets,
    state: &'a mut Editor,
    seen: Seen,
    change: Option<Change>,
}

impl BarPage {
    pub(super) fn widgets_tab(&mut self, ui: &mut Ui, c: &Colors) {
        let mut cx = Ctx {
            c,
            theme: self.theme,
            saved: &self.saved.widgets,
            state: &mut self.columns,
            seen: Seen::default(),
            change: None,
        };
        let edit = &mut self.edit.widgets;
        ui.columns(2, |columns| {
            for (ui, zone) in columns.iter_mut().zip(BarZone::ALL) {
                column(ui, &mut cx, zone, edit);
            }
        });
        drag(ui, &mut cx);
        let (state, change) = (cx.state, cx.change);
        match change {
            Some(Change::Move { from, to, by_key }) => {
                apply_move(ui, state, edit, from, to, by_key)
            }
            Some(Change::Add(zone, kind)) => {
                edit.zone_mut(zone).push(BarWidget::new(kind, true));
                state.open = Some((zone, edit.zone(zone).len() - 1));
            }
            None => {}
        }
        ui.add_space(16.0);
    }
}

fn apply_move(
    ui: &Ui,
    state: &mut Editor,
    edit: &mut BarWidgets,
    (zone, index): (BarZone, usize),
    (to, at): (BarZone, usize),
    by_key: bool,
) {
    let room = edit.zone(to).len() - usize::from(zone == to);
    let at = at.min(room);
    edit.move_to(zone, index, to, at);
    if state.open == Some((zone, index)) {
        state.open = Some((to, at));
    }
    if by_key {
        state.focus = Some(grip_id(ui, to, at));
    }
}

fn grip_id(ui: &Ui, zone: BarZone, index: usize) -> Id {
    ui.scope_id().with(("grip", zone.name(), index))
}

fn column(ui: &mut Ui, cx: &mut Ctx, zone: BarZone, edit: &mut BarWidgets) {
    let (title, note) = match zone {
        BarZone::Left => ("Left", "from the left edge"),
        BarZone::Right => ("Right", "from the right edge"),
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.label(
            RichText::new(title)
                .font(theme::bold(13.0))
                .color(cx.c.dim_text),
        );
        ui.label(
            RichText::new(note)
                .font(theme::regular(11.0))
                .color(cx.c.faint_text),
        );
    });
    ui.add_space(2.0);
    let card = Frame::new()
        .fill(cx.c.card)
        .stroke(Stroke::new(1.0, cx.c.line))
        .corner_radius(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            let count = edit.zone(zone).len();
            if count == 0 {
                ui.add_space(ROW);
            }
            for index in 0..count {
                row(
                    ui,
                    cx,
                    zone,
                    index,
                    &mut edit.zone_mut(zone)[index],
                    index + 1 == count,
                );
            }
        });
    cx.seen.columns.push((zone, card.response.rect));
    ui.add_space(6.0);
    let add = widgets::ghost_button(ui, cx.c, "Add widget");
    Popup::from_toggle_button_response(&add).show(|ui| {
        ui.set_min_width(140.0);
        for kind in WidgetKind::ALL {
            if ui.button(label(kind)).clicked() {
                cx.change = Some(Change::Add(zone, kind));
            }
        }
    });
}

/// While a grip is dragged, shows where it would land, and moves the
/// widget there when it is let go.
fn drag(ui: &mut Ui, cx: &mut Ctx) {
    let Some(from) = cx.state.drag else {
        return;
    };
    let Some(pointer) = ui.input(|i| i.pointer.latest_pos()) else {
        return;
    };
    let Some(&(zone, column)) = cx.seen.columns.iter().min_by(|(_, a), (_, b)| {
        let away = |r: &Rect| (r.center().x - pointer.x).abs();
        away(a).total_cmp(&away(b))
    }) else {
        return;
    };
    let rows: Vec<Rect> = cx
        .seen
        .rows
        .iter()
        .filter(|(z, ..)| *z == zone)
        .map(|(.., r)| *r)
        .collect();
    let at = rows.iter().filter(|r| r.center().y < pointer.y).count();
    if cx.seen.released {
        cx.state.drag = None;
        let at = if from.0 == zone && from.1 < at {
            at - 1
        } else {
            at
        };
        if (zone, at) != from {
            cx.change = Some(Change::Move {
                from,
                to: (zone, at),
                by_key: false,
            });
        }
        return;
    }
    let y = match rows.get(at) {
        Some(r) => r.top(),
        None => rows.last().map_or(column.top() + 2.0, Rect::bottom),
    };
    let line = Rect::from_min_max(
        pos2(column.left() + 6.0, y - 1.0),
        pos2(column.right() - 6.0, y + 1.0),
    );
    ui.painter().rect_filled(line, 1.0, cx.c.accent);
}
