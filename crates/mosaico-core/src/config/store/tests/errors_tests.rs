use std::path::{Path, PathBuf};

use crate::Error;
use crate::config::rules::RulesFile;

use super::migrate::migrate_config_if_needed;
use super::{load_toml, read_file};

/// A fresh directory for one test.
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-loader-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The file a config error is about, if it is a config error.
fn path_of(error: &Error) -> Option<&Path> {
    match error {
        Error::Config { path, .. } => path.as_deref(),
        _ => None,
    }
}

#[test]
fn a_file_that_cannot_be_read_is_a_config_error_about_it() {
    // Arrange
    let dir = temp_dir("unreadable");
    let path = dir.join("rules.toml");

    // Act
    let error = read_file(&path).unwrap_err();

    // Assert
    assert_eq!(path_of(&error), Some(path.as_path()), "{error:?}");
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}: ", path.display())),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_toml_is_a_config_error_about_the_file() {
    // Arrange
    let dir = temp_dir("invalid");
    let path = dir.join("rules.toml");
    std::fs::write(&path, "[[rule]\nmanage = true\n").unwrap();

    // Act
    let error = load_toml::<RulesFile>(&path).unwrap_err();

    // Assert -- worded as doctor and the config watcher have always
    // shown it: the file, then what is wrong with it.
    assert_eq!(path_of(&error), Some(path.as_path()), "{error:?}");
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}: TOML parse error", path.display())),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_migration_that_cannot_run_names_the_file() {
    // Arrange -- `workspaces` is a value, so the legacy section has
    // nowhere to go.
    let dir = temp_dir("migration");
    let path = dir.join("config.toml");
    let content = "workspaces = 3\n\n[layout.workspaces]\n1 = \"bsp\"\n";
    std::fs::write(&path, content).unwrap();

    // Act
    let error = migrate_config_if_needed(&path, content).unwrap_err();

    // Assert
    assert_eq!(path_of(&error), Some(path.as_path()), "{error:?}");
    assert_eq!(
        error.to_string(),
        format!(
            "{}: [workspaces] is not a table -- cannot migrate",
            path.display()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}
