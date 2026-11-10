use crate::config::BarChoice;
use crate::config::bar_section::read_bar_section;
use crate::config::store::test_support::{cleanup, unique_temp_path};

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_bar_provider_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_config_without_bar_gets_the_hint_and_keeps_the_built_in_bar() {
    // Arrange
    let original = "[layout]\ngap = 8\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint changes nothing.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{BAR_PROVIDER_HINT_SNIPPET}"));
    let section = read_bar_section(&on_disk).unwrap();
    assert_eq!(section.choice(), BarChoice::Mosaico);
}

#[test]
fn a_config_with_a_bar_section_is_left_alone() {
    // Arrange
    let active = "[bar]\nprovider = \"none\"\n";
    let commented = "# [bar]\n# provider = \"external\"\n";

    // Act
    let (active_appended, active_on_disk) = hinted(active);
    let (commented_appended, commented_on_disk) = hinted(commented);

    // Assert
    assert!(!active_appended);
    assert_eq!(active_on_disk, active);
    assert!(!commented_appended);
    assert_eq!(commented_on_disk, commented);
}

#[test]
fn the_bar_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "gap = 8").unwrap();
    assert!(append_missing_bar_provider_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_bar_provider_hint(&path).unwrap();

    // Assert
    assert!(!appended);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}
