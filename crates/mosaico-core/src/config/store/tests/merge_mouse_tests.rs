use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::{Config, MouseConfig, template};

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_drag_to_swap_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_config_without_drag_to_swap_gets_the_hint_and_keeps_the_defaults() {
    // Arrange -- with a [mouse] section of its own.
    let original = "[mouse]\nfollows_focus = false\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint changes nothing.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{DRAG_TO_SWAP_HINT_SNIPPET}"));
    let config: Config = toml::from_str(&on_disk).unwrap();
    let expected = MouseConfig {
        follows_focus: false,
        ..MouseConfig::default()
    };
    assert_eq!(config.mouse, expected);
}

#[test]
fn the_drag_to_swap_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "[layout]\ngap = 8\n").unwrap();
    assert!(append_missing_drag_to_swap_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_drag_to_swap_hint(&path).unwrap();

    // Assert
    assert!(!appended);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}

#[test]
fn a_config_that_mentions_drag_to_swap_gets_no_hint() {
    // Arrange -- as a key, as a dotted key, and commented out.
    let originals = [
        "[mouse]\ndrag_to_swap = false\n",
        "mouse.drag_to_swap = true\n",
        "[mouse]\n# drag_to_swap = false\n",
    ];

    // Act / Assert
    for original in originals {
        let (appended, on_disk) = hinted(original);
        assert!(!appended, "{original:?}");
        assert_eq!(on_disk, original, "the file must be untouched");
    }
}

#[test]
fn the_drag_to_swap_hint_names_no_default_value() {
    // Act -- it stays in the file when the defaults change.
    let named = DRAG_TO_SWAP_HINT_SNIPPET.to_lowercase();

    // Assert
    assert!(!named.contains("default"), "{DRAG_TO_SWAP_HINT_SNIPPET}");
}

#[test]
fn a_config_written_from_the_template_gets_no_drag_to_swap_hint() {
    // Arrange
    let original = template::generate_config();

    // Act
    let (appended, on_disk) = hinted(&original);

    // Assert
    assert!(!appended);
    assert_eq!(on_disk, original);
}

#[test]
fn the_drag_to_swap_hint_starts_on_a_line_of_its_own() {
    // Arrange -- no newline at the end.
    let original = "[layout]\ngap = 8";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert
    assert!(appended);
    assert!(on_disk.starts_with("[layout]\ngap = 8\n"), "{on_disk}");
    assert!(on_disk.contains("\n# Added automatically"), "{on_disk}");
}

#[test]
fn a_config_that_is_not_valid_toml_gets_no_drag_to_swap_hint() {
    // Arrange
    let path = unique_temp_path();
    let original = "[mouse\ngap = 8\n";
    std::fs::write(&path, original).unwrap();

    // Act
    let result = append_missing_drag_to_swap_hint(&path);

    // Assert -- the error names the file, which is left as it was.
    let error = result.unwrap_err().to_string();
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    cleanup(&path);
}
