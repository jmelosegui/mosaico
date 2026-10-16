//! A bar reload, then a theme change.

use super::tests::{MOCHA, bar, left, start};
use crate::bar::fakes::BarEvent::Drawn;
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::bar::{BarItem, BarState};
use crate::config::{BarConfig, CatppuccinFlavor, Theme};

/// Latte's green. Mocha's is `#a6e3a1`.
const LATTE_GREEN: &str = "#40a02b";

#[test]
fn a_reloaded_bar_keeps_its_height_and_widget_colors_through_a_theme_change() {
    // Arrange -- review of #109: the bars start with the default bar under
    // Mocha. bar.toml is then reloaded with a new height and a green
    // paused widget, and the daemon resolves its colors at once.
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );
    manager.reload(bar(r#"
        height = 40
        left = []
        right = [{ type = "paused", color = "green" }]
    "#));
    manager.resolve_colors(MOCHA);
    renderer.take();

    // Act
    manager.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Latte));
    manager.update(&[BarState {
        paused: true,
        ..BarState::default()
    }]);

    // Assert -- the reloaded bar, in Latte's green.
    assert_eq!(manager.bar_height(), 40);
    let drawn: Vec<Vec<BarItem>> = renderer
        .take()
        .into_iter()
        .filter_map(|event| match event {
            Drawn { content, .. } => Some(content.right),
            _ => None,
        })
        .collect();
    let paused = BarItem::Pill {
        label: "PAUSED".into(),
        text_color: LATTE_GREEN.into(),
        border_color: LATTE_GREEN.into(),
    };
    assert_eq!(drawn, [vec![paused]]);
}
