use crate::config::schema::SettingValue;
use crate::config::store::test_support::{contents, temp_store};

const FILE: &str = "\
# My setup\n\
\n\
[layout]\n\
gap = 4 # tight\n\
ratio = 0.5\n\
\n\
# Colors I like\n\
[borders]\n\
width = 2\n\
my_own_key = \"kept\"\n";

fn int(n: i64) -> SettingValue {
    SettingValue::Int(n)
}

#[test]
fn setting_a_value_changes_only_it_and_keeps_its_comment() {
    // Arrange
    let store = temp_store("document-set");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.set("layout.gap", &int(12)).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = FILE.replace("gap = 4 # tight", "gap = 12 # tight");
    assert_eq!(contents(&store.config_path()), expected);
    assert_eq!(store.load_config().layout.gap, 12);
}

#[test]
fn a_setting_the_file_leaves_out_is_added_in_its_table() {
    // Arrange
    let store = temp_store("document-add");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.set("layout.hiding", &SettingValue::Text("hide".into()))
        .unwrap();
    doc.set("mouse.drag_to_swap", &SettingValue::Bool(false))
        .unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.config_path());
    assert!(text.starts_with("# My setup\n"), "{text}");
    assert!(text.contains("ratio = 0.5\nhiding = \"hide\"\n"), "{text}");
    assert!(text.ends_with("[mouse]\ndrag_to_swap = false\n"), "{text}");
    let config = store.load_config();
    assert!(!config.mouse.drag_to_swap);
    assert_eq!(config.borders.width, 2);
}

#[test]
fn a_nested_setting_gets_its_own_table() {
    // Arrange
    let store = temp_store("document-nested");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    let red = SettingValue::Text("red".into());
    doc.set("borders.colors.focused", &red).unwrap();
    doc.save().unwrap();

    // Assert
    let text = contents(&store.config_path());
    assert!(
        text.contains("[borders.colors]\nfocused = \"red\"\n"),
        "{text}"
    );
    assert_eq!(doc.get("borders.colors.focused"), Some(red));
}

#[test]
fn a_reset_setting_is_removed_so_it_takes_its_default() {
    // Arrange
    let store = temp_store("document-reset");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.reset("borders.width");
    doc.reset("mouse.drag_to_swap");
    doc.save().unwrap();

    // Assert
    let expected = FILE.replace("width = 2\n", "");
    assert_eq!(contents(&store.config_path()), expected);
    assert_eq!(doc.get("borders.width"), None);
}

#[test]
fn a_missing_file_opens_empty_and_is_created_with_only_what_was_set() {
    // Arrange
    let store = temp_store("document-missing");
    let mut doc = store.open_settings().unwrap();

    // Act
    let before = doc.get("layout.gap");
    doc.set("layout.gap", &int(0)).unwrap();
    doc.save().unwrap();

    // Assert
    assert_eq!(before, None);
    assert_eq!(contents(&store.config_path()), "[layout]\ngap = 0\n");
}

#[test]
fn crlf_line_endings_and_a_byte_order_mark_are_kept() {
    // Arrange
    let store = temp_store("document-crlf");
    let original = format!("\u{feff}{}", FILE.replace('\n', "\r\n"));
    std::fs::write(store.config_path(), &original).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.set("layout.gap", &int(6)).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = original.replace("gap = 4", "gap = 6");
    assert_eq!(contents(&store.config_path()), expected);
}

#[test]
fn an_unknown_key_or_a_value_it_does_not_take_is_refused() {
    // Arrange
    let store = temp_store("document-refused");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    let unknown = doc.set("layout.nothing", &int(1));
    let out_of_range = doc.set("layout.gap", &int(999));
    let wrong_kind = doc.set("layout.gap", &SettingValue::Bool(true));

    // Assert
    assert!(unknown.is_err());
    assert!(out_of_range.unwrap_err().to_string().contains("0 to 200"));
    assert!(wrong_kind.is_err());
    assert!(!doc.is_dirty());
    assert_eq!(doc.get("layout.gap"), Some(int(4)));
}

#[test]
fn a_file_the_loader_would_refuse_is_not_saved() {
    // Arrange
    let store = temp_store("document-invalid");
    let broken = "[animations]\neasing = \"wobble\"\n";
    std::fs::write(store.config_path(), broken).unwrap();
    let mut doc = store.open_settings().unwrap();
    doc.set("layout.gap", &int(1)).unwrap();

    // Act
    let saved = doc.save();

    // Assert
    assert!(saved.is_err());
    assert_eq!(contents(&store.config_path()), broken);
    assert!(doc.is_dirty());
}

#[test]
fn a_file_that_is_not_toml_does_not_open() {
    let store = temp_store("document-not-toml");
    std::fs::write(store.config_path(), "[layout\n").unwrap();
    let error = store.open_settings().unwrap_err().to_string();
    assert!(error.contains("config.toml"), "{error}");
}

#[test]
fn setting_what_the_file_has_already_is_no_edit() {
    let store = temp_store("document-same");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();
    doc.set("layout.gap", &int(4)).unwrap();
    doc.reset("layout.hiding");
    assert!(!doc.is_dirty());
}

#[test]
fn a_value_put_at_its_default_is_left_out_of_the_file() {
    // Arrange
    let store = temp_store("document-put");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.put("layout.gap", &int(20)).unwrap();
    doc.put("borders.width", &int(6)).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = FILE
        .replace("gap = 4 # tight\n", "")
        .replace("width = 2", "width = 6");
    assert_eq!(contents(&store.config_path()), expected);
}

#[test]
fn the_value_in_effect_is_the_file_s_as_loaded_or_the_default() {
    let store = temp_store("document-value");
    std::fs::write(store.config_path(), "[layout]\ngap = 900\nratio = 1\n").unwrap();
    let doc = store.open_settings().unwrap();

    assert_eq!(doc.value("layout.gap"), Some(int(200)));
    assert_eq!(doc.value("layout.ratio"), Some(SettingValue::Float(0.9)));
    assert_eq!(doc.value("borders.width"), Some(int(4)));
    assert_eq!(doc.value("layout.nothing"), None);
}

#[test]
fn a_write_by_something_else_is_noticed_until_the_next_save() {
    // Arrange
    let store = temp_store("document-on-disk");
    std::fs::write(store.config_path(), FILE).unwrap();
    let mut doc = store.open_settings().unwrap();
    let unchanged = doc.has_changed_on_disk();

    // Act
    std::fs::write(store.config_path(), "[layout]\ngap = 30\n").unwrap();
    let changed = doc.has_changed_on_disk();
    doc.set("layout.gap", &int(5)).unwrap();
    doc.save().unwrap();

    // Assert
    assert!(!unchanged);
    assert!(changed);
    assert!(!doc.has_changed_on_disk());
}

#[test]
fn a_workspace_layout_is_an_entry_that_default_removes() {
    // Arrange
    let store = temp_store("document-workspace-layout");
    let file = "[workspaces.layouts] # table\n2 = \"bsp\"\n3 = \"bsp\" # wide\n";
    std::fs::write(store.config_path(), file).unwrap();
    let mut doc = store.open_settings().unwrap();
    let text = |s: &str| SettingValue::Text(s.into());

    // Act
    doc.put("workspaces.layouts.1", &text("vertical-stack"))
        .unwrap();
    doc.put("workspaces.layouts.2", &text("")).unwrap();
    doc.save().unwrap();

    // Assert
    let written = contents(&store.config_path());
    assert_eq!(
        written,
        "[workspaces.layouts] # table\n3 = \"bsp\" # wide\n1 = \"vertical-stack\"\n"
    );
    let config = store.load_config();
    assert_eq!(config.workspaces.layouts.len(), 2);
    assert_eq!(doc.value("workspaces.layouts.2"), Some(text("")));
    assert_eq!(
        doc.value("workspaces.layouts.1"),
        Some(text("vertical-stack"))
    );
}

#[test]
fn a_workspace_layout_is_added_in_a_table_of_its_own() {
    // Arrange
    let store = temp_store("document-workspace-layout-new");
    let mut doc = store.open_settings().unwrap();

    // Act
    doc.put("workspaces.layouts.8", &SettingValue::Text("bsp".into()))
        .unwrap();
    doc.save().unwrap();

    // Assert
    assert_eq!(
        contents(&store.config_path()),
        "[workspaces.layouts]\n8 = \"bsp\"\n"
    );
    assert_eq!(store.load_config().workspaces.layouts.len(), 1);
}
