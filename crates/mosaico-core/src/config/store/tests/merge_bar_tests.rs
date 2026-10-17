use crate::config::WidgetConfig;
use crate::config::store::merge_bar_mouse::BAR_MOUSE_HINT_SNIPPET;
use crate::config::store::test_support::{contents, file_names, temp_store};

use super::*;

/// The `type` of each widget, as `bar.toml` names it.
fn types(widgets: &[WidgetConfig]) -> Vec<String> {
    widgets
        .iter()
        .map(|w| {
            toml::Value::try_from(w).unwrap()["type"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn without_a_file_the_default_widgets_are_used_and_nothing_is_created() {
    // Arrange
    let store = temp_store("merge-bar-missing");

    // Act
    let bar = store.merge_missing_bar_widgets();

    // Assert
    let defaults = BarConfig::default();
    assert_eq!(types(&bar.left), types(&defaults.left));
    assert_eq!(types(&bar.right), types(&defaults.right));
    assert!(file_names(&store).is_empty(), "{:?}", file_names(&store));
}

#[test]
fn widget_types_a_side_lacks_are_appended_and_returned() {
    // Arrange -- one widget on each side, the clock with its own format.
    let store = temp_store("merge-bar-old");
    let original =
        "[[left]]\ntype = \"workspaces\"\n\n[[right]]\ntype = \"clock\"\nformat = \"%H:%M\"\n";
    std::fs::write(store.bar_path(), original).unwrap();

    // Act
    let bar = store.merge_missing_bar_widgets();

    // Assert -- the user's widgets stay first, configured as they were.
    assert_eq!(types(&bar.left), ["workspaces", "active_window", "layout"]);
    assert_eq!(
        types(&bar.right),
        ["clock", "paused", "date", "ram", "cpu", "update"]
    );
    assert!(matches!(&bar.right[0], WidgetConfig::Clock { format, .. } if format == "%H:%M"));
    let on_disk = contents(&store.bar_path());
    assert!(on_disk.starts_with(original), "{on_disk}");
    let reloaded = store.try_load_bar().unwrap();
    assert_eq!(types(&reloaded.left), types(&bar.left));
    assert_eq!(types(&reloaded.right), types(&bar.right));
}

#[test]
fn merging_bar_widgets_twice_appends_nothing_the_second_time() {
    // Arrange
    let store = temp_store("merge-bar-twice");
    std::fs::write(store.bar_path(), "[[left]]\ntype = \"layout\"\n").unwrap();
    store.merge_missing_bar_widgets();
    let merged = contents(&store.bar_path());

    // Act
    store.merge_missing_bar_widgets();

    // Assert
    assert_eq!(contents(&store.bar_path()), merged);
}

#[test]
fn an_invalid_bar_file_is_left_alone_and_the_defaults_used() {
    // Arrange
    let store = temp_store("merge-bar-invalid");
    std::fs::write(store.bar_path(), "not = [valid").unwrap();

    // Act
    let bar = store.merge_missing_bar_widgets();

    // Assert
    assert_eq!(types(&bar.left), types(&BarConfig::default().left));
    assert_eq!(contents(&store.bar_path()), "not = [valid");
}

#[test]
fn a_bar_file_with_a_value_the_bar_cannot_take_gets_the_hint_and_the_defaults_used() {
    // Arrange -- TOML, but with a height that is not a number.
    let store = temp_store("merge-bar-bad-value");
    std::fs::write(store.bar_path(), "height = \"tall\"\n").unwrap();

    // Act
    let bar = store.merge_missing_bar_widgets();

    // Assert
    assert_eq!(bar.height, BarConfig::default().height);
    let hinted = format!("height = \"tall\"\n{BAR_MOUSE_HINT_SNIPPET}");
    assert_eq!(contents(&store.bar_path()), hinted);
}
