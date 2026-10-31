//! Styles egui like the ArtCraft apps, in the colors of mosaico's theme.

mod colors;

use std::sync::Arc;

use eframe::egui::{
    Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke, Style,
    TextStyle, Visuals, vec2,
};

pub use colors::Colors;

const INTER: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");

/// The family of Inter SemiBold, for labels and headings.
fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

/// Inter SemiBold at `size` points.
pub fn bold(size: f32) -> FontId {
    FontId::new(size, bold_family())
}

/// Inter Regular at `size` points.
pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Makes Inter the only font, regular for text and SemiBold for
/// [`bold`].
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::empty();
    let data = [("inter", INTER), ("inter-semibold", INTER_SEMIBOLD)];
    for (name, bytes) in data {
        let font = Arc::new(FontData::from_static(bytes));
        fonts.font_data.insert(name.into(), font);
    }
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.insert(family, vec!["inter".into()]);
    }
    fonts
        .families
        .insert(bold_family(), vec!["inter-semibold".into(), "inter".into()]);
    ctx.set_fonts(fonts);
}

/// Paints every egui widget in `c`.
pub fn apply(ctx: &Context, c: &Colors) {
    ctx.all_styles_mut(|style| style_with(style, c));
}

fn style_with(style: &mut Style, c: &Colors) {
    let mut v = if c.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.override_text_color = Some(c.text);
    v.panel_fill = c.page;
    v.window_fill = c.card;
    v.extreme_bg_color = c.control;
    v.text_edit_bg_color = Some(c.control);
    v.faint_bg_color = c.hover;
    v.hyperlink_color = c.accent;
    v.window_stroke = Stroke::new(1.0, c.line);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = c.accent.gamma_multiply(0.85);
    v.selection.stroke = Stroke::new(1.0, c.on_accent);
    let shadow = |y, blur, alpha| Shadow {
        offset: [0, y],
        blur,
        spread: 0,
        color: eframe::egui::Color32::from_black_alpha(alpha),
    };
    v.window_shadow = shadow(4, 18, if c.dark { 110 } else { 40 });
    v.popup_shadow = shadow(3, 12, if c.dark { 100 } else { 35 });

    let radius = CornerRadius::same(5);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = c.card;
    w.noninteractive.weak_bg_fill = c.card;
    w.noninteractive.bg_stroke = Stroke::new(1.0, c.line);
    w.noninteractive.fg_stroke = Stroke::new(1.0, c.text);
    for (state, fill, width) in [
        (&mut w.inactive, c.control, 1.0),
        (&mut w.hovered, c.hover, 1.5),
        (&mut w.active, c.pressed, 1.5),
        (&mut w.open, c.pressed, 1.0),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::NONE;
        state.fg_stroke = Stroke::new(width, c.text);
        state.corner_radius = radius;
        state.expansion = 0.0;
    }
    w.noninteractive.corner_radius = radius;
    style.visuals = v;

    style.spacing.item_spacing = vec2(6.0, 4.0);
    style.spacing.button_padding = vec2(10.0, 4.0);
    style.spacing.interact_size.y = 26.0;
    style.text_styles = [
        (TextStyle::Small, regular(11.0)),
        (TextStyle::Body, regular(13.0)),
        (TextStyle::Button, regular(13.0)),
        (TextStyle::Monospace, regular(12.0)),
        (TextStyle::Heading, bold(18.0)),
    ]
    .into();
}
