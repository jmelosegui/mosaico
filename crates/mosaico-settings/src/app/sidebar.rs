//! The navigation: the search field, the pages and whether the daemon
//! runs.

use eframe::egui::{
    Align, Align2, Color32, CursorIcon, Layout, Rect, Role, Sense, Ui, WidgetInfo, vec2,
};

use super::DaemonState;
use crate::icons;
use crate::pages::Page;
use crate::theme::{self, Colors};

/// What the navigation shows.
pub struct Nav<'a> {
    /// The page shown, or shown again when the search is cleared. It is
    /// not highlighted while searching, as the results span the pages.
    pub page: Page,
    /// While searching, how many rows of each page match, in the order of
    /// `Page::ALL`.
    pub counts: Option<&'a [usize; Page::ALL.len()]>,
    pub daemon: &'a DaemonState,
}

/// Draws the navigation, editing the search `query`. Returns the page
/// clicked.
pub fn show(ui: &mut Ui, c: &Colors, nav: &Nav, query: &mut String) -> Option<Page> {
    super::search::field(ui, c, query);
    ui.add_space(12.0);
    ui.spacing_mut().item_spacing.y = 2.0;
    let mut picked = None;
    for (n, item) in Page::ALL.into_iter().enumerate() {
        let found = nav.counts.map(|counts| counts[n]);
        let selected = item == nav.page && found.is_none();
        if nav_item(ui, c, item, selected, found).clicked() {
            picked = Some(item);
        }
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        status(ui, c, nav.daemon)
    });
    picked
}

/// A page of the navigation. While searching, `found` says how many of
/// its rows match; a page with none is dimmed.
fn nav_item(
    ui: &mut Ui,
    c: &Colors,
    page: Page,
    selected: bool,
    found: Option<usize>,
) -> eframe::egui::Response {
    let ready = page.is_ready();
    let lit = found.is_none_or(|n| n > 0);
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
    let (icon, text, font) = match (selected, ready && lit) {
        (true, _) => (c.accent, c.text, theme::bold(13.0)),
        (false, true) => (c.dim_text, c.dim_text, theme::regular(13.0)),
        (false, false) => (c.faint_text, c.faint_text, theme::regular(13.0)),
    };
    let glyph = Rect::from_center_size(rect.left_center() + vec2(18.0, 0.0), vec2(16.0, 16.0));
    icons::paint(painter, glyph, page.icon(), icon, 1.5);
    let at = rect.left_center() + vec2(36.0, 0.0);
    painter.text(at, Align2::LEFT_CENTER, page.label(), font, text);
    if let Some(n) = found.filter(|&n| n > 0) {
        let at = rect.right_center() - vec2(10.0, 0.0);
        let count = n.to_string();
        painter.text(
            at,
            Align2::RIGHT_CENTER,
            count,
            theme::regular(11.0),
            c.faint_text,
        );
    }
    if ready {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response.on_hover_text("Coming in a later release")
    }
}

fn status(ui: &mut Ui, c: &Colors, daemon: &DaemonState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let version = concat!("mosaico ", env!("CARGO_PKG_VERSION"));
    let (dot, label, detail) = match daemon {
        DaemonState::Running { version } => {
            (c.good, "Daemon running", format!("mosaico {version}"))
        }
        DaemonState::Stopped => (c.faint_text, "Daemon stopped", version.to_owned()),
    };
    ui.add_space(4.0);
    let line = |ui: &mut Ui, text: &str, font, color| {
        let (rect, shown) =
            ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
        shown.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));
        let at = rect.left_center() + vec2(26.0, 0.0);
        let align = Align2::LEFT_CENTER;
        ui.painter().text(at, align, text, font, color);
        rect
    };
    line(ui, &detail, theme::regular(11.0), c.faint_text);
    let rect = line(ui, label, theme::regular(12.0), c.text);
    let dot_at = rect.left_center() + vec2(12.0, 0.0);
    ui.painter().circle_filled(dot_at, 4.0, dot);
    ui.add_space(-2.0);
}
