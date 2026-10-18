use crate::config::BarConfig;
use crate::config::bar::BarMouseConfig;
use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::template;

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_bar_mouse_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_bar_config_without_a_mouse_section_gets_the_hint_and_keeps_the_defaults() {
    // Arrange
    let original = "height = 40\n\n[[left]]\ntype = \"workspaces\"\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint changes nothing.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{BAR_MOUSE_HINT_SNIPPET}"));
    let config: BarConfig = toml::from_str(&on_disk).unwrap();
    assert_eq!(config.mouse, BarMouseConfig::default());
    assert_eq!((config.height, config.left.len()), (40, 1));
}

#[test]
fn the_mouse_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "height = 40\n").unwrap();
    assert!(append_missing_bar_mouse_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_bar_mouse_hint(&path).unwrap();

    // Assert
    assert!(!appended);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}

#[test]
fn a_bar_config_with_a_mouse_section_gets_no_hint() {
    // Arrange -- as a table, as dotted keys, commented out, and with
    // spaces in its header.
    let originals = [
        "[mouse]\nenabled = false\n",
        "mouse.scroll = \"none\"\n",
        "height = 40\n\n# [mouse]\n# enabled = false\n",
        "#[ mouse ]\n",
    ];

    // Act / Assert
    for original in originals {
        let (appended, on_disk) = hinted(original);
        assert!(!appended, "{original:?}");
        assert_eq!(on_disk, original, "the file must be untouched");
    }
}

#[test]
fn a_bar_config_that_only_mentions_the_section_gets_the_hint() {
    // Arrange -- in a comment, and in a string.
    let originals = [
        "# see [mouse] in the docs\nheight = 40\n",
        "separator = \"[mouse]\"\n",
    ];

    // Act / Assert
    for original in originals {
        let (appended, _) = hinted(original);
        assert!(appended, "{original:?}");
    }
}

#[test]
fn a_file_without_a_last_newline_gets_one_before_the_hint() {
    // Act
    let (appended, on_disk) = hinted("height = 40");

    // Assert
    assert!(appended);
    assert!(on_disk.starts_with("height = 40\n\n# Added automatically"));
}

#[test]
fn the_template_has_the_section_so_mosaico_init_files_get_no_hint() {
    // Act
    let (appended, _) = hinted(&template::generate_bar());

    // Assert
    assert!(!appended);
}

#[test]
fn a_file_that_is_not_toml_is_reported_and_left_alone() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "height = = 40\n").unwrap();

    // Act
    let result = append_missing_bar_mouse_hint(&path);

    // Assert
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "height = = 40\n");
    cleanup(&path);
}
