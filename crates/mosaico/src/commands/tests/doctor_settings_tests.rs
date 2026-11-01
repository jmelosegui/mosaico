use mosaico_core::config::schema::{BAR_SETTINGS, CONFIG_SETTINGS};

use super::*;
use crate::commands::test_support::Harness;

#[test]
fn a_value_out_of_range_is_named_as_the_settings_window_names_it() {
    // Arrange
    let h = Harness::new("doctor-settings-range");
    std::fs::create_dir_all(h.config.dir()).unwrap();
    let path = h.config.config_path();
    std::fs::write(&path, "[layout]\ngap = 5000\nratio = 0.5\n").unwrap();

    // Act
    let lines = settings_lines("config.toml", &path, CONFIG_SETTINGS);
    let _ = std::fs::remove_dir_all(h.config.dir());

    // Assert
    assert_eq!(
        lines,
        [format!(
            "  {WARN} config.toml: Gap (layout.gap) is 5000; it takes 0 to 200 px"
        )]
    );
}

#[test]
fn a_missing_file_has_nothing_to_say() {
    let h = Harness::new("doctor-settings-missing");

    let lines = settings_lines("bar.toml", &h.config.bar_path(), BAR_SETTINGS);

    assert!(lines.is_empty(), "{lines:?}");
}

#[test]
fn doctor_says_an_unknown_bar_key_right_after_the_bar_toml_check() {
    // Arrange
    let mut h = Harness::new("doctor-settings-execute");
    std::fs::create_dir_all(h.config.dir()).unwrap();
    std::fs::write(h.config.bar_path(), "heigth = 30\n").unwrap();

    // Act
    h.run(crate::commands::doctor::execute);
    let _ = std::fs::remove_dir_all(h.config.dir());

    // Assert
    let expected =
        format!("bar.toml is valid\n  {WARN} bar.toml: heigth is not a setting mosaico knows\n");
    assert!(h.out().contains(&expected), "{}", h.out());
}
