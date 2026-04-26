//! Catppuccin color palettes (Mocha, Macchiato, Frappé, Latte).
//!
//! See <https://catppuccin.com/palette/> for the canonical hex values.

use crate::config::bar::BarColors;
use crate::config::theme::CatppuccinFlavor;

pub(super) fn table(flavor: CatppuccinFlavor) -> &'static [(&'static str, &'static str)] {
    match flavor {
        CatppuccinFlavor::Mocha => MOCHA,
        CatppuccinFlavor::Macchiato => MACCHIATO,
        CatppuccinFlavor::Frappe => FRAPPE,
        CatppuccinFlavor::Latte => LATTE,
    }
}

pub(super) fn bar_colors(flavor: CatppuccinFlavor) -> BarColors {
    match flavor {
        CatppuccinFlavor::Mocha => mocha_bar(),
        CatppuccinFlavor::Macchiato => macchiato_bar(),
        CatppuccinFlavor::Frappe => frappe_bar(),
        CatppuccinFlavor::Latte => latte_bar(),
    }
}

const MOCHA: &[(&str, &str)] = &[
    ("rosewater", "#f5e0dc"),
    ("flamingo", "#f2cdcd"),
    ("pink", "#f5c2e7"),
    ("mauve", "#cba6f7"),
    ("red", "#f38ba8"),
    ("maroon", "#eba0ac"),
    ("peach", "#fab387"),
    ("yellow", "#f9e2af"),
    ("green", "#a6e3a1"),
    ("teal", "#94e2d5"),
    ("sky", "#89dceb"),
    ("sapphire", "#74c7ec"),
    ("blue", "#89b4fa"),
    ("lavender", "#b4befe"),
];

const MACCHIATO: &[(&str, &str)] = &[
    ("rosewater", "#f4dbd6"),
    ("flamingo", "#f0c6c6"),
    ("pink", "#f5bde6"),
    ("mauve", "#c6a0f6"),
    ("red", "#ed8796"),
    ("maroon", "#ee99a0"),
    ("peach", "#f5a97f"),
    ("yellow", "#eed49f"),
    ("green", "#a6da95"),
    ("teal", "#8bd5ca"),
    ("sky", "#91d7e3"),
    ("sapphire", "#7dc4e4"),
    ("blue", "#8aadf4"),
    ("lavender", "#b7bdf8"),
];

const FRAPPE: &[(&str, &str)] = &[
    ("rosewater", "#f2d5cf"),
    ("flamingo", "#eebebe"),
    ("pink", "#f4b8e4"),
    ("mauve", "#ca9ee6"),
    ("red", "#e78284"),
    ("maroon", "#ea999c"),
    ("peach", "#ef9f76"),
    ("yellow", "#e5c890"),
    ("green", "#a6d189"),
    ("teal", "#81c8be"),
    ("sky", "#99d1db"),
    ("sapphire", "#85c1dc"),
    ("blue", "#8caaee"),
    ("lavender", "#babbf1"),
];

const LATTE: &[(&str, &str)] = &[
    ("rosewater", "#dc8a78"),
    ("flamingo", "#dd7878"),
    ("pink", "#ea76cb"),
    ("mauve", "#8839ef"),
    ("red", "#d20f39"),
    ("maroon", "#e64553"),
    ("peach", "#fe640b"),
    ("yellow", "#df8e1d"),
    ("green", "#40a02b"),
    ("teal", "#179299"),
    ("sky", "#04a5e5"),
    ("sapphire", "#209fb5"),
    ("blue", "#1e66f5"),
    ("lavender", "#7287fd"),
];

fn mocha_bar() -> BarColors {
    BarColors {
        background: "#1e1e2e".into(),
        foreground: "#89b4fa".into(),
        active_workspace: "#435375".into(),
        active_workspace_text: "#cdd6f4".into(),
        inactive_workspace: "#89b4fa".into(),
        separator: "#45475a".into(),
        accent: "#a6e3a1".into(),
        widget_background: "#313244".into(),
        pill_border: "#89b4fa".into(),
    }
}

fn macchiato_bar() -> BarColors {
    BarColors {
        background: "#24273a".into(),
        foreground: "#8aadf4".into(),
        active_workspace: "#48567b".into(),
        active_workspace_text: "#cad3f5".into(),
        inactive_workspace: "#8aadf4".into(),
        separator: "#494d64".into(),
        accent: "#a6da95".into(),
        widget_background: "#363a4f".into(),
        pill_border: "#8aadf4".into(),
    }
}

fn frappe_bar() -> BarColors {
    BarColors {
        background: "#303446".into(),
        foreground: "#8caaee".into(),
        active_workspace: "#505d81".into(),
        active_workspace_text: "#c6d0f5".into(),
        inactive_workspace: "#8caaee".into(),
        separator: "#51576d".into(),
        accent: "#a6d189".into(),
        widget_background: "#414559".into(),
        pill_border: "#8caaee".into(),
    }
}

fn latte_bar() -> BarColors {
    BarColors {
        background: "#eff1f5".into(),
        foreground: "#1e66f5".into(),
        active_workspace: "#7287d5".into(),
        active_workspace_text: "#eff1f5".into(),
        inactive_workspace: "#1e66f5".into(),
        separator: "#bcc0cc".into(),
        accent: "#40a02b".into(),
        widget_background: "#ccd0da".into(),
        pill_border: "#1e66f5".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_flavor_has_14_named_colors() {
        assert_eq!(MOCHA.len(), 14);
        assert_eq!(MACCHIATO.len(), 14);
        assert_eq!(FRAPPE.len(), 14);
        assert_eq!(LATTE.len(), 14);
    }

    #[test]
    fn mocha_blue_matches_catppuccin_palette() {
        let blue = MOCHA.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
        assert_eq!(blue, Some("#89b4fa"));
    }
}
