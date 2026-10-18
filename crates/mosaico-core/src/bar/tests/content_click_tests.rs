//! Which items of a bar take clicks, and the ones with no region.

use crate::bar::fakes::FakeSystem;
use crate::bar::{BarContent, BarItem, BarSide, BarState, BarTarget, SystemReadings};
use crate::config::BarConfig;

use super::support::{AT, bar, content, widget};

/// Whether the item at `index` of `side` in `content` takes clicks.
fn takes_clicks(content: &BarContent, side: BarSide, index: usize) -> bool {
    content.region(side, index, AT).is_some_and(|r| r.clickable)
}

#[test]
fn an_index_past_the_items_has_no_region() {
    // Arrange
    let config = bar("left = [{ type = \"layout\" }]\nright = []");

    // Act
    let content = content(&config);

    // Assert
    assert_eq!(content.region(BarSide::Left, 1, AT), None);
    assert_eq!(content.region(BarSide::Right, 0, AT), None);
}

#[test]
fn workspace_labels_take_clicks_and_widgets_do_not() {
    // Arrange
    let config = BarConfig::default();

    // Act
    let content = content(&config);

    // Assert
    let label = content.label_region(3, AT);
    assert_eq!(label.target, BarTarget::Workspace(3));
    assert!(label.clickable);
    assert!(
        content
            .region(BarSide::Left, 0, AT)
            .is_some_and(|r| !r.clickable)
    );
}

#[test]
fn with_the_mouse_off_the_labels_take_no_clicks() {
    // Arrange
    let config = bar("[mouse]\nenabled = false");

    // Act
    let content = content(&config);

    // Assert
    assert!(!content.label_region(0, AT).clickable);
}

#[test]
fn on_the_right_the_workspaces_widget_is_a_pill_with_no_labels() {
    // Arrange -- the right side's workspaces pill takes no clicks.
    let config = bar("left = []\nright = [{ type = \"workspaces\" }]");

    // Act
    let content = content(&config);

    // Assert
    assert!(matches!(content.right[0], BarItem::Pill { .. }));
    assert_eq!(
        content.region(BarSide::Right, 0, AT),
        widget(BarSide::Right, 0)
    );
}

#[test]
fn by_default_the_layout_and_paused_widgets_take_clicks() {
    // Arrange -- the paused widget shows while the hotkeys are paused.
    let config = BarConfig::default();
    let state = BarState {
        paused: true,
        ..BarState::default()
    };
    let readings = SystemReadings::read(&FakeSystem::default(), &config, None);

    // Act
    let content = BarContent::build(&config, &state, &readings);

    // Assert -- workspaces, the window icon and layout on the left;
    // paused, clock, date, ram and cpu on the right.
    assert!(takes_clicks(&content, BarSide::Left, 2));
    assert!(takes_clicks(&content, BarSide::Right, 0));
    for (side, index) in [(BarSide::Left, 0), (BarSide::Left, 1), (BarSide::Right, 1)] {
        assert!(!takes_clicks(&content, side, index), "{side:?} {index}");
    }
}

#[test]
fn a_widget_a_binding_names_takes_clicks() {
    // Arrange
    let config = bar(r#"
        left = []
        right = [{ type = "date" }, { type = "clock" }]

        [[mouse.click]]
        widget = "clock"
        button = "middle"
        action = "retile"
    "#);

    // Act
    let content = content(&config);

    // Assert
    assert!(takes_clicks(&content, BarSide::Right, 1));
    assert!(!takes_clicks(&content, BarSide::Right, 0));
}

#[test]
fn with_the_mouse_off_no_widget_takes_clicks() {
    // Arrange
    let config = bar("[mouse]\nenabled = false");

    // Act
    let content = content(&config);

    // Assert
    assert!(!takes_clicks(&content, BarSide::Left, 2));
}

#[test]
fn a_binding_on_the_workspaces_widget_makes_its_container_take_clicks_on_the_left() {
    // Arrange
    let config = bar(r#"
        right = [{ type = "workspaces" }]

        [[mouse.click]]
        widget = "workspaces"
        button = "right"
        action = "cycle-layout"
    "#);

    // Act
    let content = content(&config);

    // Assert -- the pill on the right takes none.
    assert!(takes_clicks(&content, BarSide::Left, 0));
    assert!(!takes_clicks(&content, BarSide::Right, 0));
}
