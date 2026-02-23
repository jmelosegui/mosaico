//! Catppuccin color palettes for each theme flavor.
//!
//! Provides the full named-color palette and the [`BarColors`]
//! mapping for each theme.

use super::bar::BarColors;
use super::theme::Theme;

/// Resolves a Catppuccin color name (e.g. "blue", "green") to its
/// hex value for the given theme. Returns `None` for unknown names.
pub fn named_color(theme: Theme, name: &str) -> Option<&'static str> {
    let table = match theme {
        Theme::Mocha => MOCHA,
        Theme::Macchiato => MACCHIATO,
        Theme::Frappe => FRAPPE,
        Theme::Latte => LATTE,
    };
    table
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, hex)| *hex)
}

/// Returns the bar color palette for the given theme.
pub fn bar_colors(theme: Theme) -> BarColors {
    match theme {
        Theme::Mocha => mocha_bar(),
        Theme::Macchiato => macchiato_bar(),
        Theme::Frappe => frappe_bar(),
        Theme::Latte => latte_bar(),
    }
}

// -- Named color tables (14 accent colors per flavor) ----------------

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

// -- Bar color mappings per theme ------------------------------------

fn mocha_bar() -> BarColors {
    BarColors {
        background: "#1e1e2e".into(),
        foreground: "#89b4fa".into(),
        active_workspace: "#435375".into(),
        active_workspace_text: "#cdd6f4".into(),
        inactive_workspace: "#89b4fa".into(),
        separator: "#45475a".into(),
        accent: "#f38ba8".into(),
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
        accent: "#ed8796".into(),
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
        accent: "#e78284".into(),
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
        accent: "#d20f39".into(),
        widget_background: "#ccd0da".into(),
        pill_border: "#1e66f5".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_color_resolves_blue() {
        assert_eq!(named_color(Theme::Mocha, "blue"), Some("#89b4fa"));
        assert_eq!(named_color(Theme::Latte, "blue"), Some("#1e66f5"));
    }

    #[test]
    fn named_color_is_case_insensitive() {
        assert_eq!(named_color(Theme::Mocha, "Blue"), Some("#89b4fa"));
        assert_eq!(named_color(Theme::Mocha, "GREEN"), Some("#a6e3a1"));
    }

    #[test]
    fn named_color_returns_none_for_unknown() {
        assert_eq!(named_color(Theme::Mocha, "chartreuse"), None);
        assert_eq!(named_color(Theme::Mocha, "#89b4fa"), None);
    }

    #[test]
    fn each_theme_has_14_named_colors() {
        assert_eq!(MOCHA.len(), 14);
        assert_eq!(MACCHIATO.len(), 14);
        assert_eq!(FRAPPE.len(), 14);
        assert_eq!(LATTE.len(), 14);
    }
}
