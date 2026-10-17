//! Painting a bar on a canvas of the test's own, in memory: where its
//! items go, and the opacity of its pixels (`render_alpha_tests.rs`).

use mosaico_core::bar::{BarSide, BarState, BarTarget, SystemReadings, WidgetRef};
use mosaico_core::config::{CatppuccinFlavor, Theme};

use super::super::canvas::Canvas;
use super::*;

const WIDTH: i32 = 1920;

/// The default bar, colored as the default theme colors it, and what it
/// shows on a monitor with 8 workspaces and nothing focused.
fn default_bar() -> (BarConfig, BarContent) {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));
    let state = BarState::default();
    let readings = SystemReadings::default();
    let content = BarContent::build(&config, &state, &readings);
    (config, content)
}

/// Paints the default bar on a canvas, and returns it with the regions.
pub(super) fn painted(takes_input: bool) -> Option<(Canvas, Vec<ItemRegion>)> {
    let (config, content) = default_bar();
    let mut canvas = Canvas::new(WIDTH, config.height).ok()?;
    let regions = paint(&mut canvas, &config, &content, takes_input);
    Some((canvas, regions))
}

#[test]
fn every_item_and_label_gets_a_region_in_the_order_drawn() {
    // Arrange
    let Some((_canvas, regions)) = painted(true) else {
        return;
    };

    // Act
    let targets: Vec<BarTarget> = regions.iter().map(|r| r.target).collect();

    // Assert -- the workspaces container, its 8 labels, the window icon
    // and the layout pill on the left; clock, date, ram and cpu on the
    // right, where the paused and update widgets do not show.
    let widget = |side, index| BarTarget::Widget(WidgetRef { side, index });
    let mut expected = vec![widget(BarSide::Left, 0)];
    expected.extend((0..8).map(BarTarget::Workspace));
    expected.extend([widget(BarSide::Left, 1), widget(BarSide::Left, 2)]);
    expected.extend((1..5).map(|index| widget(BarSide::Right, index)));
    assert_eq!(targets, expected);
}

#[test]
fn labels_lie_in_their_container_left_to_right_the_full_height_of_the_bar() {
    // Arrange
    let Some((canvas, regions)) = painted(true) else {
        return;
    };

    // Act
    let container = regions[0].rect;
    let labels: Vec<Rect> = regions[1..9].iter().map(|r| r.rect).collect();

    // Assert
    for pair in labels.windows(2) {
        assert!(pair[0].x + pair[0].width <= pair[1].x, "{pair:?}");
    }
    for label in &labels {
        assert!(label.x >= container.x && label.x + label.width <= container.x + container.width);
        assert_eq!((label.y, label.height), (0, canvas.h));
    }
    assert!(regions[1..9].iter().all(|r| r.clickable));
}

#[test]
fn the_right_side_ends_its_padding_in_from_the_right_edge() {
    // Arrange
    let Some((_canvas, regions)) = painted(true) else {
        return;
    };
    let padding = BarConfig::default().padding;

    // Act
    let first_on_the_right = regions[11].rect;

    // Assert
    assert_eq!(
        first_on_the_right.x + first_on_the_right.width,
        WIDTH - padding
    );
}

#[test]
fn an_items_region_stops_where_the_gap_after_it_begins() {
    // Arrange
    let Some((_canvas, regions)) = painted(true) else {
        return;
    };
    let gap = BarConfig::default().item_gap;

    // Act -- the workspaces container, then the layout pill: the window
    // icon between them draws nothing with no window focused.
    let (container, pill) = (regions[0].rect, regions[10].rect);

    // Assert
    assert_eq!(container.x + container.width + gap, pill.x);
}

#[path = "render_alpha_tests.rs"]
mod alpha_tests;
