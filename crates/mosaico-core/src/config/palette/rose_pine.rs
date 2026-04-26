//! Rosé Pine color palettes (Main, Moon, Dawn).
//!
//! See <https://rosepinetheme.com/palette/> for the canonical hex
//! values. Each flavor exposes Rosé Pine's six accent colors (love,
//! gold, rose, pine, foam, iris) plus common aliases (red, yellow,
//! green, blue) so portable configs like `focused = "blue"` resolve
//! to a Rosé Pine shade rather than nothing.

use crate::config::bar::BarColors;
use crate::config::theme::RosePineFlavor;

pub(super) fn table(flavor: RosePineFlavor) -> &'static [(&'static str, &'static str)] {
    match flavor {
        RosePineFlavor::Main => MAIN,
        RosePineFlavor::Moon => MOON,
        RosePineFlavor::Dawn => DAWN,
    }
}

pub(super) fn bar_colors(flavor: RosePineFlavor) -> BarColors {
    match flavor {
        RosePineFlavor::Main => main_bar(),
        RosePineFlavor::Moon => moon_bar(),
        RosePineFlavor::Dawn => dawn_bar(),
    }
}

const MAIN: &[(&str, &str)] = &[
    // Rosé Pine native accents
    ("love", "#eb6f92"),
    ("gold", "#f6c177"),
    ("rose", "#ebbcba"),
    ("pine", "#31748f"),
    ("foam", "#9ccfd8"),
    ("iris", "#c4a7e7"),
    // Common color aliases for portable configs
    ("red", "#eb6f92"),     // alias for love
    ("yellow", "#f6c177"),  // alias for gold
    ("green", "#9ccfd8"),   // alias for foam
    ("blue", "#31748f"),    // alias for pine
    ("teal", "#9ccfd8"),    // alias for foam
    ("mauve", "#c4a7e7"),   // alias for iris
    ("purple", "#c4a7e7"),  // alias for iris
    ("magenta", "#c4a7e7"), // alias for iris
];

const MOON: &[(&str, &str)] = &[
    ("love", "#eb6f92"),
    ("gold", "#f6c177"),
    ("rose", "#ea9a97"),
    ("pine", "#3e8fb0"),
    ("foam", "#9ccfd8"),
    ("iris", "#c4a7e7"),
    ("red", "#eb6f92"),
    ("yellow", "#f6c177"),
    ("green", "#9ccfd8"),
    ("blue", "#3e8fb0"),
    ("teal", "#9ccfd8"),
    ("mauve", "#c4a7e7"),
    ("purple", "#c4a7e7"),
    ("magenta", "#c4a7e7"),
];

const DAWN: &[(&str, &str)] = &[
    ("love", "#b4637a"),
    ("gold", "#ea9d34"),
    ("rose", "#d7827e"),
    ("pine", "#286983"),
    ("foam", "#56949f"),
    ("iris", "#907aa9"),
    ("red", "#b4637a"),
    ("yellow", "#ea9d34"),
    ("green", "#56949f"),
    ("blue", "#286983"),
    ("teal", "#56949f"),
    ("mauve", "#907aa9"),
    ("purple", "#907aa9"),
    ("magenta", "#907aa9"),
];

// Bar identity uses Iris (purple) for foreground/pill borders and
// Rose (the namesake) for the secondary accent. Pine is reserved for
// the focus border (canonical "active" color in upstream Rose Pine
// implementations) so both Iris/Rose and Pine end up visible.

fn main_bar() -> BarColors {
    BarColors {
        background: "#191724".into(),            // Base
        foreground: "#c4a7e7".into(),            // Iris
        active_workspace: "#403d52".into(),      // Highlight Med
        active_workspace_text: "#e0def4".into(), // Text
        inactive_workspace: "#c4a7e7".into(),    // Iris
        separator: "#26233a".into(),             // Overlay
        accent: "#ebbcba".into(),                // Rose
        widget_background: "#1f1d2e".into(),     // Surface
        pill_border: "#c4a7e7".into(),           // Iris
    }
}

fn moon_bar() -> BarColors {
    BarColors {
        background: "#232136".into(),            // Base
        foreground: "#c4a7e7".into(),            // Iris
        active_workspace: "#44415a".into(),      // Highlight Med
        active_workspace_text: "#e0def4".into(), // Text
        inactive_workspace: "#c4a7e7".into(),    // Iris
        separator: "#393552".into(),             // Overlay
        accent: "#ea9a97".into(),                // Rose (Moon variant)
        widget_background: "#2a273f".into(),     // Surface
        pill_border: "#c4a7e7".into(),           // Iris
    }
}

fn dawn_bar() -> BarColors {
    BarColors {
        background: "#faf4ed".into(),            // Base
        foreground: "#907aa9".into(),            // Iris (Dawn variant)
        active_workspace: "#dfdad9".into(),      // Highlight Med
        active_workspace_text: "#575279".into(), // Text
        inactive_workspace: "#907aa9".into(),    // Iris
        separator: "#cecacd".into(),             // Highlight High
        accent: "#d7827e".into(),                // Rose (Dawn variant)
        widget_background: "#fffaf3".into(),     // Surface
        pill_border: "#907aa9".into(),           // Iris
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_flavor_has_pine_and_blue_alias() {
        for (table, name) in [(MAIN, "main"), (MOON, "moon"), (DAWN, "dawn")] {
            let pine = table.iter().find(|(n, _)| *n == "pine").map(|(_, h)| *h);
            let blue = table.iter().find(|(n, _)| *n == "blue").map(|(_, h)| *h);
            assert!(pine.is_some(), "{name}: missing pine");
            assert_eq!(pine, blue, "{name}: blue should alias pine");
        }
    }

    #[test]
    fn dawn_is_lighter_than_main() {
        // Dawn's base is the lightest of the three (off-white).
        assert_eq!(dawn_bar().background, "#faf4ed");
        // Main's base is the darkest.
        assert_eq!(main_bar().background, "#191724");
    }

    #[test]
    fn bar_lead_color_is_rose_family() {
        // The bar foreground (the most visible accent) uses Iris on
        // dark flavors and Iris-Dawn on the light flavor so the
        // Rose Pine identity reads as purple-rose, not teal.
        assert_eq!(main_bar().foreground, "#c4a7e7");
        assert_eq!(moon_bar().foreground, "#c4a7e7");
        assert_eq!(dawn_bar().foreground, "#907aa9");
    }

    #[test]
    fn bar_accent_uses_rose() {
        // The secondary accent is the namesake Rose hex per flavor.
        assert_eq!(main_bar().accent, "#ebbcba");
        assert_eq!(moon_bar().accent, "#ea9a97");
        assert_eq!(dawn_bar().accent, "#d7827e");
    }
}
