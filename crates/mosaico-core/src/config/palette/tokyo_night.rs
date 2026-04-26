//! Tokyo Night color palettes (Night, Storm, Day).
//!
//! Hex values come from the upstream Tokyo Night theme
//! (<https://github.com/folke/tokyonight.nvim>). Night and Storm
//! share their accent palette and only differ in the base background
//! and a few surface shades.

use crate::config::bar::BarColors;
use crate::config::theme::TokyoNightFlavor;

pub(super) fn table(flavor: TokyoNightFlavor) -> &'static [(&'static str, &'static str)] {
    match flavor {
        TokyoNightFlavor::Night => NIGHT,
        TokyoNightFlavor::Storm => STORM,
        TokyoNightFlavor::Day => DAY,
    }
}

pub(super) fn bar_colors(flavor: TokyoNightFlavor) -> BarColors {
    match flavor {
        TokyoNightFlavor::Night => night_bar(),
        TokyoNightFlavor::Storm => storm_bar(),
        TokyoNightFlavor::Day => day_bar(),
    }
}

const NIGHT: &[(&str, &str)] = &[
    ("red", "#f7768e"),
    ("orange", "#ff9e64"),
    ("yellow", "#e0af68"),
    ("green", "#9ece6a"),
    ("cyan", "#7dcfff"),
    ("teal", "#1abc9c"),
    ("blue", "#7aa2f7"),
    ("purple", "#9d7cd8"),
    ("magenta", "#bb9af7"),
    // Common aliases
    ("mauve", "#bb9af7"), // alias for magenta
];

const STORM: &[(&str, &str)] = &[
    ("red", "#f7768e"),
    ("orange", "#ff9e64"),
    ("yellow", "#e0af68"),
    ("green", "#9ece6a"),
    ("cyan", "#7dcfff"),
    ("teal", "#1abc9c"),
    ("blue", "#7aa2f7"),
    ("purple", "#9d7cd8"),
    ("magenta", "#bb9af7"),
    ("mauve", "#bb9af7"),
];

const DAY: &[(&str, &str)] = &[
    ("red", "#f52a65"),
    ("orange", "#b15c00"),
    ("yellow", "#8c6c3e"),
    ("green", "#587539"),
    ("cyan", "#007197"),
    ("teal", "#118c74"),
    ("blue", "#2e7de9"),
    ("purple", "#7847bd"),
    ("magenta", "#9854f1"),
    ("mauve", "#9854f1"),
];

fn night_bar() -> BarColors {
    BarColors {
        background: "#1a1b26".into(),
        foreground: "#7aa2f7".into(), // blue
        active_workspace: "#3b4261".into(),
        active_workspace_text: "#c0caf5".into(),
        inactive_workspace: "#7aa2f7".into(),
        separator: "#414868".into(),
        accent: "#9ece6a".into(), // green
        widget_background: "#16161e".into(),
        pill_border: "#7aa2f7".into(),
    }
}

fn storm_bar() -> BarColors {
    BarColors {
        background: "#24283b".into(),
        foreground: "#7aa2f7".into(),
        active_workspace: "#3b4261".into(),
        active_workspace_text: "#c0caf5".into(),
        inactive_workspace: "#7aa2f7".into(),
        separator: "#414868".into(),
        accent: "#9ece6a".into(),
        widget_background: "#1f2335".into(),
        pill_border: "#7aa2f7".into(),
    }
}

fn day_bar() -> BarColors {
    BarColors {
        background: "#e1e2e7".into(),
        foreground: "#2e7de9".into(), // upstream Day blue
        active_workspace: "#a8aecb".into(),
        active_workspace_text: "#3760bf".into(),
        inactive_workspace: "#2e7de9".into(),
        separator: "#a8aecb".into(),
        accent: "#587539".into(), // upstream Day green
        widget_background: "#d0d5e3".into(),
        pill_border: "#2e7de9".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_and_storm_share_accent_palette() {
        let night_blue = NIGHT.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
        let storm_blue = STORM.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
        assert_eq!(night_blue, storm_blue);
    }

    #[test]
    fn day_is_a_distinct_light_palette() {
        let day_blue = DAY.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
        let night_blue = NIGHT.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
        assert_ne!(day_blue, night_blue);
        assert_eq!(day_bar().background, "#e1e2e7");
    }

    #[test]
    fn purple_and_magenta_are_distinct() {
        // Upstream tokyonight.nvim has purple (#9d7cd8) and magenta
        // (#bb9af7) as separate shades.
        for table in [NIGHT, STORM] {
            let purple = table.iter().find(|(n, _)| *n == "purple").map(|(_, h)| *h);
            let magenta = table.iter().find(|(n, _)| *n == "magenta").map(|(_, h)| *h);
            assert_eq!(purple, Some("#9d7cd8"));
            assert_eq!(magenta, Some("#bb9af7"));
        }
        let day_purple = DAY.iter().find(|(n, _)| *n == "purple").map(|(_, h)| *h);
        let day_magenta = DAY.iter().find(|(n, _)| *n == "magenta").map(|(_, h)| *h);
        assert_eq!(day_purple, Some("#7847bd"));
        assert_eq!(day_magenta, Some("#9854f1"));
    }

    #[test]
    fn mauve_aliases_magenta() {
        // `mauve` is a Catppuccin name; we expose it as an alias for
        // magenta so portable configs (focused = "mauve") resolve
        // under Tokyo Night too.
        for table in [NIGHT, STORM, DAY] {
            let magenta = table.iter().find(|(n, _)| *n == "magenta").map(|(_, h)| *h);
            let mauve = table.iter().find(|(n, _)| *n == "mauve").map(|(_, h)| *h);
            assert_eq!(magenta, mauve);
        }
    }

    #[test]
    fn day_red_matches_upstream() {
        // Sanity-check a single Day value against tokyonight.nvim so
        // a future copy-paste regression is caught.
        let red = DAY.iter().find(|(n, _)| *n == "red").map(|(_, h)| *h);
        assert_eq!(red, Some("#f52a65"));
    }
}
