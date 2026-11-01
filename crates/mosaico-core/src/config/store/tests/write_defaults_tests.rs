use crate::config::store::test_support::temp_store;
use crate::config::template;

use super::*;

/// The file names in `files`, each with what happened to it.
fn outcomes(files: &[DefaultFile]) -> Vec<String> {
    files
        .iter()
        .map(|file| {
            let (what, path) = match file {
                DefaultFile::Created(path) => ("created", path),
                DefaultFile::Existed(path) => ("existed", path),
                DefaultFile::Failed(path, _) => ("failed", path),
            };
            format!("{what} {}", path.file_name().unwrap().to_string_lossy())
        })
        .collect()
}

#[test]
fn every_file_is_written_into_a_directory_that_does_not_exist_yet() {
    // Arrange
    let store = temp_store("defaults-fresh");
    std::fs::remove_dir_all(store.dir()).unwrap();

    // Act
    let files = store.write_missing_defaults().unwrap();

    // Assert
    assert_eq!(
        outcomes(&files),
        [
            "created config.toml",
            "created keybindings.toml",
            "created rules.toml",
            "created user-rules.toml",
            "created bar.toml",
        ]
    );
    let config = std::fs::read_to_string(store.config_path()).unwrap();
    assert_eq!(config, template::generate_config());
}

#[test]
fn a_file_that_exists_is_left_as_it_is() {
    // Arrange
    let store = temp_store("defaults-existing");
    std::fs::write(store.config_path(), "[layout]\ngap = 3\n").unwrap();

    // Act
    let files = store.write_missing_defaults().unwrap();

    // Assert
    assert_eq!(
        outcomes(&files)[..2],
        ["existed config.toml", "created keybindings.toml"]
    );
    let config = std::fs::read_to_string(store.config_path()).unwrap();
    assert_eq!(config, "[layout]\ngap = 3\n");
}
