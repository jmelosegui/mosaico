use std::collections::HashMap;
use std::path::Path;

use super::*;

/// Builds an environment lookup from key/value pairs.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let map: HashMap<String, OsString> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |key| map.get(key).cloned()
}

/// A platform that reports the given settings.
struct Dirs {
    config: UserDir,
    data: UserDir,
}

impl UserDirs for Dirs {
    fn config_dir(&self) -> UserDir {
        self.config.clone()
    }

    fn data_dir(&self) -> UserDir {
        self.data.clone()
    }
}

/// The setting `name`, holding `value`.
fn setting(name: &'static str, value: Option<PathBuf>) -> UserDir {
    UserDir {
        setting: name,
        value,
        subfolder: None,
    }
}

/// A home folder that is an absolute path on every system.
fn home() -> PathBuf {
    std::env::temp_dir().join("me")
}

/// The user's directories, as the platform reports them.
fn user() -> Dirs {
    Dirs {
        config: setting("CONFIG_HOME", Some(home().join("config"))),
        data: setting("DATA_HOME", Some(home().join("data"))),
    }
}

#[test]
fn the_default_instance_keeps_the_usual_paths_and_names() {
    // Act
    let instance = Instance::from_env(env(&[]), &user()).unwrap();

    // Assert
    assert!(instance.is_default());
    assert_eq!(instance.endpoint_name(), "mosaico");
    assert_eq!(instance.config_dir(), home().join("config").join("mosaico"));
    assert_eq!(
        instance.pid_path(),
        home().join("data").join("mosaico").join("mosaico.pid")
    );
    assert_eq!(instance.log_dir(), instance.config_dir().join("logs"));
}

#[test]
fn a_setting_can_name_a_folder_above_the_directory() {
    // Arrange -- such as a home folder with the configuration in .config.
    let dirs = Dirs {
        config: UserDir {
            subfolder: Some(".config"),
            ..setting("HOME", Some(home()))
        },
        ..user()
    };

    // Act
    let instance = Instance::from_env(env(&[]), &dirs).unwrap();

    // Assert
    assert_eq!(
        instance.config_dir(),
        home().join(".config").join("mosaico")
    );
}

#[test]
fn an_empty_instance_variable_means_the_default_instance() {
    // Act
    let instance = Instance::from_env(env(&[(INSTANCE_VAR, "")]), &user()).unwrap();

    // Assert
    assert!(instance.is_default());
}

#[test]
fn a_named_instance_shares_no_files_with_the_default_one() {
    // Arrange
    let default = Instance::from_env(env(&[]), &user()).unwrap();

    // Act
    let named = Instance::from_env(env(&[(INSTANCE_VAR, "test-1")]), &user()).unwrap();

    // Assert
    assert_eq!(named.name(), Some("test-1"));
    assert_eq!(named.endpoint_name(), "mosaico-test-1");
    assert_ne!(named.config_dir(), default.config_dir());
    assert_ne!(named.pid_path(), default.pid_path());
    assert!(
        named
            .config_dir()
            .ends_with(Path::new("instances").join("test-1"))
    );
}

#[test]
fn directories_can_be_given_explicitly() {
    // Arrange
    let nowhere = Dirs {
        config: setting("CONFIG_HOME", None),
        data: setting("DATA_HOME", None),
    };

    // Act
    let instance = Instance::from_env(
        env(&[
            (INSTANCE_VAR, "ci"),
            (CONFIG_DIR_VAR, "/tmp/cfg"),
            (DATA_DIR_VAR, "/tmp/data"),
        ]),
        &nowhere,
    )
    .unwrap();

    // Assert -- the platform's directories are not needed at all.
    assert_eq!(instance.config_dir(), Path::new("/tmp/cfg"));
    assert_eq!(
        instance.pid_path(),
        Path::new("/tmp/data").join("mosaico.pid")
    );
}

#[test]
fn an_override_replaces_only_its_own_directory() {
    // Act
    let config = Instance::from_env(env(&[(CONFIG_DIR_VAR, "cfg")]), &user()).unwrap();
    let data = Instance::from_env(env(&[(DATA_DIR_VAR, "data")]), &user()).unwrap();

    // Assert -- used as given, even when relative.
    assert_eq!(config.config_dir(), Path::new("cfg"));
    assert_eq!(config.data_dir(), home().join("data").join("mosaico"));
    assert_eq!(data.config_dir(), home().join("config").join("mosaico"));
    assert_eq!(data.data_dir(), Path::new("data"));
}

#[test]
fn an_empty_override_is_ignored() {
    // Act
    let instance =
        Instance::from_env(env(&[(CONFIG_DIR_VAR, ""), (DATA_DIR_VAR, "")]), &user()).unwrap();

    // Assert
    assert_eq!(instance, Instance::from_env(env(&[]), &user()).unwrap());
}

#[test]
fn a_bad_name_is_refused_rather_than_ignored() {
    // Act / Assert -- falling back to the default would reach the
    // user's own daemon.
    for bad in ["../escape", "has space", "pipe\\name", &"x".repeat(65)] {
        let result = Instance::from_env(env(&[(INSTANCE_VAR, bad)]), &user());
        assert!(
            matches!(result, Err(InstanceError::InvalidName(_))),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn a_directory_the_platform_does_not_know_is_reported() {
    // Arrange
    let no_config = Dirs {
        config: setting("CONFIG_HOME", None),
        ..user()
    };
    let no_data = Dirs {
        data: setting("DATA_HOME", None),
        ..user()
    };

    // Act / Assert -- with no override either, there is nowhere to keep
    // the files, so the CLI refuses to start and says what to set.
    assert_eq!(
        Instance::from_env(env(&[]), &no_config),
        Err(InstanceError::NoDirectory {
            dir: "config",
            override_var: CONFIG_DIR_VAR,
            reason: Unusable::Unset("CONFIG_HOME"),
        })
    );
    assert_eq!(
        Instance::from_env(env(&[]), &no_data),
        Err(InstanceError::NoDirectory {
            dir: "data",
            override_var: DATA_DIR_VAR,
            reason: Unusable::Unset("DATA_HOME"),
        })
    );
}

#[test]
fn a_setting_that_is_empty_or_not_absolute_is_refused() {
    // Arrange -- a relative base would put the files wherever mosaico
    // happens to run.
    let relative = PathBuf::from("me").join("config");
    let cases = [
        (Some(PathBuf::new()), "CONFIG_HOME is empty".to_string()),
        (
            Some(relative.clone()),
            format!(
                "CONFIG_HOME is not an absolute path ({})",
                relative.display()
            ),
        ),
    ];
    for (value, reason) in cases {
        let dirs = Dirs {
            config: setting("CONFIG_HOME", value),
            ..user()
        };

        // Act
        let result = Instance::from_env(env(&[]), &dirs);

        // Assert
        match result {
            Err(error) => assert_eq!(
                error.to_string(),
                format!(
                    "could not determine the config directory: {reason}; \
                     set MOSAICO_CONFIG_DIR to choose one"
                )
            ),
            Ok(instance) => panic!("{reason}: must be refused, got {instance:?}"),
        }
    }
}

#[test]
fn an_override_needs_no_usable_setting() {
    // Arrange
    let dirs = Dirs {
        config: setting("CONFIG_HOME", Some(PathBuf::from("rel"))),
        ..user()
    };

    // Act
    let instance = Instance::from_env(env(&[(CONFIG_DIR_VAR, "cfg")]), &dirs).unwrap();

    // Assert -- the override is used as given.
    assert_eq!(instance.config_dir(), Path::new("cfg"));
}
