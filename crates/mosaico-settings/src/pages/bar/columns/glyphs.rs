//! The icons a widget can show, as Nerd Font glyphs, with the name and
//! drawing the window shows for each.

use crate::icons::Icon;
use crate::widgets::Choice;

/// The glyphs offered for a widget's icon.
const GLYPHS: [(&str, &str, Icon); 10] = [
    ("\u{F009}", "Grid", Icon::Grid),
    ("\u{F2D0}", "Window", Icon::Window),
    ("\u{F017}", "Clock", Icon::Clock),
    ("\u{F073}", "Calendar", Icon::Calendar),
    ("\u{F2DB}", "Memory", Icon::Memory),
    ("\u{F085}", "Processor", Icon::Processor),
    ("\u{F019}", "Download", Icon::Download),
    ("\u{F04C}", "Pause", Icon::Pause),
    ("\u{F001}", "Music", Icon::Music),
    ("\u{F0DB}", "Columns", Icon::Layout),
];

/// The choices for a widget whose icon is `current`: no icon, each known
/// glyph, and `current` itself when it is a glyph not listed.
pub(super) fn choices(current: &str) -> Vec<Choice<'static, String>> {
    let mut choices = vec![(String::new(), "None", None)];
    choices.extend(
        GLYPHS
            .iter()
            .map(|&(glyph, name, icon)| (glyph.to_owned(), name, Some(icon))),
    );
    if !current.is_empty() && GLYPHS.iter().all(|(glyph, ..)| *glyph != current) {
        choices.push((current.to_owned(), "Custom", None));
    }
    choices
}

#[cfg(test)]
mod tests {
    use super::choices;

    #[test]
    fn a_known_glyph_is_offered_once() {
        let offered = choices("\u{F017}");

        assert_eq!(offered.iter().filter(|(g, ..)| g == "\u{F017}").count(), 1);
        assert!(offered.iter().all(|(_, name, _)| *name != "Custom"));
    }

    #[test]
    fn a_glyph_not_listed_is_kept_as_custom() {
        let offered = choices("X");

        assert_eq!(
            offered.last().map(|(g, name, _)| (g.as_str(), *name)),
            Some(("X", "Custom"))
        );
    }

    #[test]
    fn no_icon_is_the_first_choice() {
        assert_eq!(choices("")[0].1, "None");
        assert!(choices("").iter().all(|(_, name, _)| *name != "Custom"));
    }
}
