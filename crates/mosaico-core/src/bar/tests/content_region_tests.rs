//! The widget each item of a bar shows, as the content tells the
//! platform that draws it.

use crate::bar::{BarItem, BarSide, ItemRegion};

use super::support::{AT, bar, content, widget};

#[test]
fn each_item_names_the_entry_of_its_widget_in_the_config() {
    // Arrange
    let config = bar(r#"
        left = [{ type = "workspaces" }, { type = "active_window" }, { type = "layout" }]
        right = [{ type = "clock" }, { type = "date" }]
    "#);

    // Act
    let content = content(&config);

    // Assert -- the layout widget takes clicks, as it does by default.
    assert_eq!(
        content.region(BarSide::Left, 0, AT),
        widget(BarSide::Left, 0)
    );
    let layout = widget(BarSide::Left, 2).map(|r| ItemRegion {
        clickable: true,
        ..r
    });
    assert_eq!(content.region(BarSide::Left, 2, AT), layout);
    assert_eq!(
        content.region(BarSide::Right, 1, AT),
        widget(BarSide::Right, 1)
    );
}

#[test]
fn a_widget_that_does_not_show_keeps_the_entries_after_it_in_place() {
    // Arrange -- the update widget hides while there is no update.
    let config = bar(r#"
        left = []
        right = [{ type = "update" }, { type = "clock" }]
    "#);

    // Act
    let content = content(&config);

    // Assert
    assert_eq!(content.right.len(), 1);
    assert_eq!(
        content.region(BarSide::Right, 0, AT),
        widget(BarSide::Right, 1)
    );
}

#[test]
fn a_separator_has_no_region_and_the_widgets_around_it_keep_theirs() {
    // Arrange
    let config = bar(r#"
        separator = "|"
        right = []
        left = [{ type = "layout" }, { type = "clock" }]
    "#);

    // Act
    let content = content(&config);

    // Assert
    assert!(matches!(content.left[1], BarItem::Separator(_)));
    assert_eq!(content.region(BarSide::Left, 1, AT), None);
    assert_eq!(
        content.region(BarSide::Left, 2, AT),
        widget(BarSide::Left, 1)
    );
}

#[test]
fn two_widgets_of_the_same_type_name_their_own_entries() {
    // Arrange
    let config = bar("left = []\nright = [{ type = \"clock\" }, { type = \"clock\" }]");

    // Act
    let content = content(&config);

    // Assert
    assert_eq!(
        content.region(BarSide::Right, 0, AT),
        widget(BarSide::Right, 0)
    );
    assert_eq!(
        content.region(BarSide::Right, 1, AT),
        widget(BarSide::Right, 1)
    );
}
