//! The icons a widget can show, as Nerd Font glyphs, with the name and
//! drawing the window shows for each.

use mosaico_core::bar::BarIcon;

use crate::icons::Icon;
use crate::widgets::Choice;

/// The drawing the window shows for glyph, when it is one it knows.
pub(in crate::pages::bar) fn drawing(glyph: &str) -> Option<Icon> {
    BarIcon::from_glyph(glyph).map(Icon::Widget)
}

/// The choices for a widget whose icon is `current`: no icon, each known
/// glyph, and `current` itself when it is a glyph not listed.
pub(super) fn choices(current: &str) -> Vec<Choice<'static, String>> {
    let mut choices = vec![(String::new(), "None", None)];
    choices.extend(BarIcon::ALL.map(|icon| {
        (
            icon.glyph().to_owned(),
            icon.name(),
            Some(Icon::Widget(icon)),
        )
    }));
    if !current.is_empty() && BarIcon::from_glyph(current).is_none() {
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
