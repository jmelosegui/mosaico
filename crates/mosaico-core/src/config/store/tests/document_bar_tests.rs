use crate::config::schema::SettingValue;
use crate::config::store::test_support::{contents, temp_store};
use crate::config::{BarWidget, BarWidgets, BarZone, WidgetConfig, WidgetKind};

const FILE: &str = "\
# My bar\n\
height = 40 # tall enough\n\
\n\
[colors]\n\
accent = \"red\"\n\
\n\
# Where I am\n\
[[left]]\n\
type = \"workspaces\"\n\
\n\
# What I listen to\n\
[[left]]\n\
type = \"media\"\n\
max_length = 20\n\
\n\
[[right]]\n\
type = \"clock\"\n\
format = \"%H:%M\" # no seconds\n";

fn kinds(widgets: &[BarWidget]) -> Vec<WidgetKind> {
    widgets.iter().map(|w| w.kind).collect()
}

#[test]
fn a_bar_setting_is_set_keeping_its_comment_and_the_bar_loads_with_it() {
    // Arrange
    let store = temp_store("bar-document-set");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();

    // Act
    doc.set("height", &SettingValue::Int(48)).unwrap();
    doc.put("mouse.scroll", &SettingValue::Text("none".into()))
        .unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    assert!(text.contains("height = 48 # tall enough\n"), "{text}");
    assert!(text.contains("[mouse]\nscroll = \"none\"\n"), "{text}");
    assert_eq!(store.load_bar().height, 48);
}

#[test]
fn a_bar_setting_set_to_its_default_is_removed_and_one_out_of_range_reads_clamped() {
    // Arrange
    let store = temp_store("bar-document-default");
    std::fs::write(store.bar_path(), "height = 500\nfont_size = 30\n").unwrap();
    let mut doc = store.open_bar_settings().unwrap();

    // Act
    let clamped = doc.value("height");
    doc.put("font_size", &SettingValue::Int(24)).unwrap();
    doc.save().unwrap();

    // Assert
    assert_eq!(clamped, Some(SettingValue::Int(96)));
    assert_eq!(contents(&store.bar_path()), "height = 500\n");
    assert_eq!(doc.value("font_size"), Some(SettingValue::Int(24)));
}

#[test]
fn only_the_settings_of_the_bar_are_taken_and_only_in_range() {
    let store = temp_store("bar-document-refuse");
    let mut doc = store.open_bar_settings().unwrap();

    assert!(doc.set("layout.gap", &SettingValue::Int(4)).is_err());
    assert!(doc.set("height", &SettingValue::Int(500)).is_err());
    assert_eq!(doc.value("layout.gap"), None);
    assert!(!doc.is_dirty());
}

#[test]
fn a_bar_file_the_loader_refuses_is_not_saved() {
    // Arrange
    let store = temp_store("bar-document-broken");
    let broken = "height = \"tall\"\n";
    std::fs::write(store.bar_path(), broken).unwrap();
    let mut doc = store.open_bar_settings().unwrap();

    // Act
    doc.set("font_size", &SettingValue::Int(20)).unwrap();
    let saved = doc.save();

    // Assert
    assert!(saved.is_err());
    assert_eq!(contents(&store.bar_path()), broken);
}

#[test]
fn the_widgets_are_those_the_file_lists_or_the_defaults_of_a_side_it_leaves_out() {
    // Arrange
    let store = temp_store("bar-widgets-read");
    std::fs::write(store.bar_path(), "[[left]]\ntype = \"cpu\"\n").unwrap();

    // Act
    let widgets = store.open_bar_settings().unwrap().widgets().unwrap();

    // Assert
    assert_eq!(kinds(&widgets.left), [WidgetKind::Cpu]);
    assert_eq!(widgets.right, BarWidgets::default().right);
}

#[test]
fn a_widget_moved_keeps_its_table_and_its_comments() {
    // Arrange
    let store = temp_store("bar-widgets-move");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.move_to(BarZone::Left, 1, BarZone::Right, 0);
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    let media = "# What I listen to\n[[right]]\ntype = \"media\"\nmax_length = 20\n";
    assert!(
        text.starts_with("# My bar\nheight = 40 # tall enough\n"),
        "{text}"
    );
    assert!(
        text.contains("# Where I am\n[[left]]\ntype = \"workspaces\"\n"),
        "{text}"
    );
    assert!(text.contains(media), "{text}");
    let bar = store.load_bar();
    assert_eq!(bar.left.len(), 1);
    assert!(matches!(
        bar.right[0],
        WidgetConfig::Media { max_length: 20, .. }
    ));
    assert!(matches!(&bar.right[1], WidgetConfig::Clock { format, .. } if format == "%H:%M"));
    assert_eq!(bar.colors.accent, "red");
}

#[test]
fn widgets_reordered_in_one_side_come_back_in_that_order() {
    // Arrange
    let store = temp_store("bar-widgets-reorder");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.move_to(BarZone::Left, 0, BarZone::Left, 1);
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    let media = text.find("type = \"media\"").unwrap();
    let workspaces = text.find("type = \"workspaces\"").unwrap();
    assert!(media < workspaces, "{text}");
    let bar = store.load_bar();
    assert_eq!(
        kinds(&BarWidgets::of(&bar).left),
        [WidgetKind::Media, WidgetKind::Workspaces]
    );
}

#[test]
fn a_widget_is_disabled_and_enabled_again_without_touching_its_other_keys() {
    // Arrange
    let store = temp_store("bar-widgets-enabled");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.right[0].enabled = false;
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();
    let disabled = contents(&store.bar_path());
    widgets.right[0].enabled = true;
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    assert!(!store.load_bar().right.is_empty());
    assert!(
        disabled.contains("type = \"clock\"\nformat = \"%H:%M\" # no seconds\nenabled = false\n"),
        "{disabled}"
    );
    assert_eq!(contents(&store.bar_path()), FILE);
}

#[test]
fn a_file_without_widgets_gets_the_defaults_written_out_with_the_change() {
    // Arrange
    let store = temp_store("bar-widgets-defaults");
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();
    let default = BarWidgets::default();

    // Act
    widgets.move_to(BarZone::Right, 0, BarZone::Left, 0);
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let bar = store.load_bar();
    assert_eq!(BarWidgets::of(&bar), widgets);
    assert_eq!(bar.right.len() + 1, default.right.len());
    assert_eq!(bar.right[0].icon(), "\u{F017}");
}

#[test]
fn a_side_left_without_widgets_is_written_as_an_empty_list() {
    // Arrange
    let store = temp_store("bar-widgets-empty");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.right.clear();
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let bar = store.load_bar();
    assert!(bar.right.is_empty());
    assert_eq!(bar.left.len(), 2);
}

#[test]
fn widgets_written_as_an_inline_list_stay_inline() {
    // Arrange
    let store = temp_store("bar-widgets-inline");
    let file = "left = [{ type = \"clock\" }, { type = \"cpu\", icon = \"x\" }]\nright = []\n";
    std::fs::write(store.bar_path(), file).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.move_to(BarZone::Left, 1, BarZone::Left, 0);
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.bar_path());
    assert!(text.starts_with("left = ["), "{text}");
    assert!(!text.contains("[[left]]"), "{text}");
    let bar = store.load_bar();
    assert_eq!(bar.left[0].icon(), "x");
    assert_eq!(bar.left[1].kind(), WidgetKind::Clock);
}

#[test]
fn a_widget_the_file_lacks_gets_a_table_of_its_own() {
    // Arrange
    let store = temp_store("bar-widgets-new");
    std::fs::write(store.bar_path(), FILE).unwrap();
    let mut doc = store.open_bar_settings().unwrap();
    let mut widgets = doc.widgets().unwrap();

    // Act
    widgets.left.push(BarWidget::new(WidgetKind::Ram, true));
    doc.set_widgets(&widgets).unwrap();
    doc.save().unwrap();

    // Assert
    assert_eq!(BarWidgets::of(&store.load_bar()), widgets);
}
