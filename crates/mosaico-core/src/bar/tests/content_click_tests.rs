//! Which items of a bar take clicks, and the ones with no region.

use crate::bar::{BarItem, BarSide, BarTarget};
use crate::config::BarConfig;

use super::support::{AT, bar, content, widget};

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
