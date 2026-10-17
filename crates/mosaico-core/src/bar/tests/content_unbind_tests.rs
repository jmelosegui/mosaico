//! Widgets whose every binding `action = "none"` removes take no clicks
//! in their regions, so the pointer shows them no hand and the bar gives
//! them no least alpha.

use crate::bar::fakes::FakeSystem;
use crate::bar::{BarContent, BarSide, BarState, SystemReadings};

use super::support::{AT, bar};

/// What the default bar, with the `[[mouse.click]]` entries of `toml`,
/// shows while the hotkeys are paused: the workspaces, the window icon
/// and the layout widget on the left, and the paused widget first on the
/// right.
fn paused(toml: &str) -> BarContent {
    let config = bar(toml);
    let state = BarState {
        paused: true,
        ..BarState::default()
    };
    let readings = SystemReadings::read(&FakeSystem::default(), &config, None);
    BarContent::build(&config, &state, &readings)
}

/// Whether the item at `index` of `side` in `content` takes clicks.
fn takes_clicks(content: &BarContent, side: BarSide, index: usize) -> bool {
    content.region(side, index, AT).is_some_and(|r| r.clickable)
}

/// An entry binding `button` on `widget` to `action`.
fn entry(widget: &str, button: &str, action: &str) -> String {
    format!(
        "[[mouse.click]]\nwidget = \"{widget}\"\nbutton = \"{button}\"\naction = \"{action}\"\n"
    )
}

#[test]
fn with_every_default_removed_the_layout_and_paused_widgets_take_no_clicks() {
    // Arrange
    let toml = [
        entry("layout", "left", "none"),
        entry("layout", "right", "none"),
        entry("paused", "left", "none"),
    ]
    .concat();

    // Act
    let content = paused(&toml);

    // Assert
    assert!(!takes_clicks(&content, BarSide::Left, 2));
    assert!(!takes_clicks(&content, BarSide::Right, 0));
}

#[test]
fn a_widget_keeps_taking_clicks_while_one_of_its_bindings_is_left() {
    // Act
    let content = paused(&entry("layout", "left", "none"));

    // Assert -- its right click still toggles monocle.
    assert!(takes_clicks(&content, BarSide::Left, 2));
}

#[test]
fn a_none_that_a_later_entry_overrides_leaves_the_widget_clickable() {
    // Arrange -- and the other way round on the clock.
    let toml = [
        entry("paused", "left", "none"),
        entry("paused", "left", "toggle-pause"),
        entry("clock", "middle", "retile"),
        entry("clock", "middle", "none"),
    ]
    .concat();

    // Act
    let content = paused(&toml);

    // Assert -- the clock comes second on the right, after the paused widget.
    assert!(takes_clicks(&content, BarSide::Right, 0));
    assert!(!takes_clicks(&content, BarSide::Right, 1));
}
