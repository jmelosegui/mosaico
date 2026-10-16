use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::{AnimationConfig, Config, template};

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_animations_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_config_without_animations_gets_the_hint_and_keeps_the_defaults() {
    // Arrange
    let original = "[layout]\ngap = 8\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint turns nothing on.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{ANIMATIONS_HINT_SNIPPET}"));
    let config: Config = toml::from_str(&on_disk).unwrap();
    assert_eq!(config.animations, AnimationConfig::default());
    assert_eq!(config.layout.gap, 8);
}

#[test]
fn the_animations_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "[layout]\ngap = 8\n").unwrap();
    assert!(append_missing_animations_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_animations_hint(&path).unwrap();

    // Assert
    assert!(!appended);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}

#[test]
fn a_config_with_an_animations_section_gets_no_hint() {
    // Arrange -- written as a table, as dotted keys, and commented out,
    // with spaces in its header or a comment after it.
    let originals = [
        "[animations]\nenabled = true\n",
        "animations.enabled = true\n",
        "[layout]\ngap = 8\n\n# [animations]\n# enabled = true\n",
        "##[ animations ] # transitions\n#enabled = true\n",
    ];

    // Act / Assert
    for original in originals {
        let (appended, on_disk) = hinted(original);
        assert!(!appended, "{original:?}");
        assert_eq!(on_disk, original, "the file must be untouched");
    }
}

#[test]
fn a_config_that_only_mentions_the_section_gets_the_hint() {
    // Arrange -- the near miss: in a comment, and in a string.
    let originals = [
        "# see [animations] in the docs\n[layout]\ngap = 8\n",
        "[theme]\nname = \"[animations]\"\n",
    ];

    // Act / Assert
    for original in originals {
        let (appended, on_disk) = hinted(original);
        assert!(appended, "{original:?}");
        assert!(on_disk.starts_with(original), "{on_disk}");
    }
}

#[test]
fn the_animations_hint_names_no_default_value() {
    // Act -- it stays in the file when the defaults change.
    let named = ANIMATIONS_HINT_SNIPPET.to_lowercase();

    // Assert
    assert!(!named.contains("default"), "{ANIMATIONS_HINT_SNIPPET}");
}

#[test]
fn a_config_written_from_the_template_gets_no_hint() {
    // Arrange
    let original = template::generate_config();

    // Act
    let (appended, on_disk) = hinted(&original);

    // Assert
    assert!(!appended);
    assert_eq!(on_disk, original);
}

#[test]
fn the_animations_hint_starts_on_a_line_of_its_own() {
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
fn a_config_that_is_not_valid_toml_gets_no_hint() {
    // Arrange
    let path = unique_temp_path();
    let original = "[layout\ngap = 8\n";
    std::fs::write(&path, original).unwrap();

    // Act
    let result = append_missing_animations_hint(&path);

    // Assert -- the error names the file, which is left as it was.
    let error = result.unwrap_err().to_string();
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    cleanup(&path);
}
