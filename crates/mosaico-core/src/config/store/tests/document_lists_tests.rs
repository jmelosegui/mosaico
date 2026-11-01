use crate::config::store::test_support::{contents, temp_store};
use crate::config::{BarWidgets, BarZone, ClickBinding, WidgetKind};

#[test]
fn the_monitors_of_the_bar_are_written_and_none_removes_the_key() {
    // Arrange
    let store = temp_store("bar-monitors");
    std::fs::write(store.bar_path(), "height = 40\n").unwrap();
    let mut doc = store.open_bar_settings().unwrap();

    // Act
    doc.set_monitors(&[1]).unwrap();
    doc.save().unwrap();
    let text = contents(&store.bar_path());
    let loaded = store.load_bar().monitors;
    let mut doc = store.open_bar_settings().unwrap();
    doc.set_monitors(&[]).unwrap();
    doc.save().unwrap();

    // Assert
    assert!(text.contains("monitors = [1]"), "{text}");
    assert_eq!(loaded, [1]);
    assert!(!contents(&store.bar_path()).contains("monitors"));
    assert!(store.load_bar().monitors.is_empty());
}

#[test]
fn click_bindings_are_written_as_what_differs_from_the_defaults() {
    // Arrange
    let store = temp_store("bar-clicks");
    let mut doc = store.open_bar_settings().unwrap();
    let rows = vec![
        ClickBinding::new("layout", "left", "retile"),
        ClickBinding::new("paused", "left", "toggle-pause"),
        ClickBinding::new("workspaces", "middle", "retile"),
    ];

    // Act
    doc.set_clicks(&rows).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    assert!(text.contains("[[mouse.click]]"), "{text}");
    assert!(text.contains("action = \"none\""), "{text}");
    assert_eq!(store.open_bar_settings().unwrap().clicks().unwrap(), rows);
}

#[test]
fn the_default_click_bindings_leave_no_entries_in_the_file() {
    // Arrange
    let store = temp_store("bar-clicks-default");
    let mut doc = store.open_bar_settings().unwrap();
    doc.set_clicks(&[ClickBinding::new("clock", "left", "retile")])
        .unwrap();
    doc.save().unwrap();

    // Act
    let mut doc = store.open_bar_settings().unwrap();
    doc.set_clicks(&ClickBinding::defaults()).unwrap();
    doc.save().unwrap();

    // Assert
    assert!(!contents(&store.bar_path()).contains("click"));
}

#[test]
fn a_widgets_format_icon_and_color_are_written_keeping_its_comments() {
    // Arrange
    let store = temp_store("bar-widget-options");
    std::fs::write(
        store.bar_path(),
        "[[right]]\ntype = \"clock\"\nformat = \"%H:%M\" # no seconds\n",
    )
    .unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    let clock = &mut widgets.zone_mut(BarZone::Right)[0];
    assert_eq!(clock.kind, WidgetKind::Clock);
    clock.format = Some("%H".into());
    clock.icon = "X".into();
    clock.color = "red".into();
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    assert!(text.contains("format = \"%H\" # no seconds"), "{text}");
    assert!(text.contains("icon = \"X\""), "{text}");
    assert_eq!(BarWidgets::of(&store.load_bar()), widgets);
}
