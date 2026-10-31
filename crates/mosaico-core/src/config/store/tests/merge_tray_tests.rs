use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::{Config, TrayConfig};

use super::*;

/// Writes `original` to a fresh file, appends the hint to it and returns
/// whether it did, and what the file holds then.
fn hinted(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let appended = append_missing_tray_hint(&path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (appended, on_disk)
}

#[test]
fn a_config_without_tray_gets_the_hint_and_keeps_the_icon() {
    // Arrange
    let original = "[layout]\ngap = 8\n";

    // Act
    let (appended, on_disk) = hinted(original);

    // Assert -- every byte is kept, and the hint changes nothing.
    assert!(appended);
    assert_eq!(on_disk, format!("{original}{TRAY_HINT_SNIPPET}"));
    let config: Config = toml::from_str(&on_disk).unwrap();
    assert_eq!(config.tray, TrayConfig::default());
    assert!(config.tray.enabled);
}

#[test]
fn a_config_with_a_tray_section_is_left_alone() {
    // Arrange
    let active = "[tray]\nenabled = false\n";
    let commented = "# [ tray ]\n# enabled = false\n";

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
fn the_tray_hint_is_appended_once() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "gap = 8").unwrap();
    assert!(append_missing_tray_hint(&path).unwrap());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Act
    let appended = append_missing_tray_hint(&path).unwrap();

    // Assert -- the first run started the block on a line of its own.
    assert!(!appended);
    assert!(after_first.starts_with("gap = 8\n"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    cleanup(&path);
}
