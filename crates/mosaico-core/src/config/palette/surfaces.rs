//! The neutral colors of each flavor, darkest to lightest in dark
//! flavors and the reverse in light ones, from which [`super::ui_colors`]
//! picks the colors of an app's surfaces and text.

use crate::config::theme::{CatppuccinFlavor, RosePineFlavor, Theme, TokyoNightFlavor};

/// A flavor's neutrals, named as Catppuccin names them. Families with
/// other names map theirs onto these.
pub(super) struct Surfaces {
    pub dark: bool,
    pub crust: &'static str,
    pub mantle: &'static str,
    pub base: &'static str,
    pub surface0: &'static str,
    pub surface1: &'static str,
    pub surface2: &'static str,
    pub overlay: &'static str,
    pub subtext: &'static str,
    pub text: &'static str,
}

/// `[crust, mantle, base, surface0, surface1, surface2, overlay, subtext, text]`.
type Ramp = [&'static str; 9];

const MOCHA: Ramp = [
    "#11111b", "#181825", "#1e1e2e", "#313244", "#45475a", "#585b70", "#7f849c", "#a6adc8",
    "#cdd6f4",
];
const MACCHIATO: Ramp = [
    "#181926", "#1e2030", "#24273a", "#363a4f", "#494d64", "#5b6078", "#8087a2", "#a5adcb",
    "#cad3f5",
];
const FRAPPE: Ramp = [
    "#232634", "#292c3c", "#303446", "#414559", "#51576d", "#626880", "#838ba7", "#a5adce",
    "#c6d0f5",
];
const LATTE: Ramp = [
    "#dce0e8", "#e6e9ef", "#eff1f5", "#ccd0da", "#bcc0cc", "#acb0be", "#8c8fa1", "#6c6f85",
    "#4c4f69",
];
// Rosé Pine: Base, Base, Surface, Overlay, Highlight Med, Highlight High,
// Muted, Subtle, Text. Dawn: Overlay, Base, Surface, then the same.
const MAIN: Ramp = [
    "#191724", "#191724", "#1f1d2e", "#26233a", "#403d52", "#524f67", "#6e6a86", "#908caa",
    "#e0def4",
];
const MOON: Ramp = [
    "#232136", "#232136", "#2a273f", "#393552", "#44415a", "#56526e", "#6e6a86", "#908caa",
    "#e0def4",
];
const DAWN: Ramp = [
    "#f2e9e1", "#faf4ed", "#fffaf3", "#dfdad9", "#cecacd", "#cecacd", "#9893a5", "#797593",
    "#575279",
];
// Tokyo Night: bg_dark, bg_dark, bg, bg_highlight, fg_gutter, terminal
// black, comment, fg_dark, fg. Day: bg_highlight, bg_dark, bg, then the same.
const NIGHT: Ramp = [
    "#16161e", "#16161e", "#1a1b26", "#292e42", "#3b4261", "#414868", "#565f89", "#a9b1d6",
    "#c0caf5",
];
const STORM: Ramp = [
    "#1f2335", "#1f2335", "#24283b", "#292e42", "#3b4261", "#414868", "#565f89", "#a9b1d6",
    "#c0caf5",
];
const DAY: Ramp = [
    "#c4c8da", "#d0d5e3", "#e1e2e7", "#c4c8da", "#a8aecb", "#a1a6c5", "#848cb5", "#6172b0",
    "#3760bf",
];

/// The neutrals of `theme`.
pub(super) fn surfaces(theme: Theme) -> Surfaces {
    let (ramp, dark) = match theme {
        Theme::Catppuccin(f) => match f {
            CatppuccinFlavor::Mocha => (MOCHA, true),
            CatppuccinFlavor::Macchiato => (MACCHIATO, true),
            CatppuccinFlavor::Frappe => (FRAPPE, true),
            CatppuccinFlavor::Latte => (LATTE, false),
        },
        Theme::RosePine(f) => match f {
            RosePineFlavor::Main => (MAIN, true),
            RosePineFlavor::Moon => (MOON, true),
            RosePineFlavor::Dawn => (DAWN, false),
        },
        Theme::TokyoNight(f) => match f {
            TokyoNightFlavor::Night => (NIGHT, true),
            TokyoNightFlavor::Storm => (STORM, true),
            TokyoNightFlavor::Day => (DAY, false),
        },
    };
    let [
        crust,
        mantle,
        base,
        surface0,
        surface1,
        surface2,
        overlay,
        subtext,
        text,
    ] = ramp;
    Surfaces {
        dark,
        crust,
        mantle,
        base,
        surface0,
        surface1,
        surface2,
        overlay,
        subtext,
        text,
    }
}
