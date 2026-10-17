use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::{Config, MouseConfig, template};

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_resize_splits_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_config_without_resize_splits_gets_the_hint_and_keeps_the_defaults() {
    // Arrange -- with the other new [mouse] key, which is not this one.
    let original = "[mouse]\nfollows_focus = false\ndrag_to_swap = true\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint changes nothing.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{RESIZE_SPLITS_HINT_SNIPPET}"));
    let config: Config = toml::from_str(&on_disk).unwrap();
    let expected = MouseConfig {
        follows_focus: false,
        ..MouseConfig::default()
    };
    assert_eq!(config.mouse, expected);
}

#[test]
fn the_resize_splits_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "[layout]\ngap = 8").unwrap();
    assert!(append_missing_resize_splits_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_resize_splits_hint(&path).unwrap();

    // Assert -- on a line of its own, and once.
    assert!(!appended);
    assert!(after_first.starts_with("[layout]\ngap = 8\n\n# Added"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}

#[test]
fn a_config_that_mentions_resize_splits_gets_no_hint() {
    // Arrange -- as a key, as a dotted key, commented out, and the
    // template.
    let template = template::generate_config();
    let originals = [
        "[mouse]\nresize_splits = false\n",
        "mouse.resize_splits = true\n",
        "[mouse]\n# resize_splits = false\n",
        template.as_str(),
    ];

    // Act / Assert
    for original in originals {
        let (appended, on_disk) = hinted(original);
        assert!(!appended, "{original:?}");
        assert_eq!(on_disk, original, "the file must be untouched");
    }
}

#[test]
fn the_resize_splits_hint_names_no_default_value() {
    // Act -- it stays in the file when the defaults change.
    let named = RESIZE_SPLITS_HINT_SNIPPET.to_lowercase();

    // Assert
    assert!(!named.contains("default"), "{RESIZE_SPLITS_HINT_SNIPPET}");
}

#[test]
fn a_config_that_is_not_valid_toml_gets_no_resize_splits_hint() {
    // Arrange
    let path = unique_temp_path();
    let original = "[mouse\ngap = 8\n";
    std::fs::write(&path, original).unwrap();

    // Act
    let result = append_missing_resize_splits_hint(&path);

    // Assert -- the error names the file, which is left as it was.
    let error = result.unwrap_err().to_string();
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    cleanup(&path);
}
