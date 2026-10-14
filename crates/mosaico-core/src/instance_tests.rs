use std::collections::HashMap;

use super::*;

/// Builds an environment lookup from key/value pairs.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let map: HashMap<String, OsString> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |key| map.get(key).cloned()
}

const HOME: (&str, &str) = ("USERPROFILE", r"C:\Users\me");
const APPDATA: (&str, &str) = ("LOCALAPPDATA", r"C:\Users\me\AppData\Local");

#[test]
fn the_default_instance_keeps_the_usual_paths_and_names() {
    // Act
    let instance = Instance::from_env(env(&[HOME, APPDATA])).unwrap();

    // Assert
    assert!(instance.is_default());
    assert_eq!(instance.endpoint_name(), "mosaico");
    assert_eq!(
        instance.config_dir(),
        Path::new(r"C:\Users\me").join(".config").join("mosaico")
    );
    assert_eq!(
        instance.pid_path(),
        Path::new(r"C:\Users\me\AppData\Local")
            .join("mosaico")
            .join("mosaico.pid")
    );
    assert_eq!(instance.log_dir(), instance.config_dir().join("logs"));
}

#[test]
fn an_empty_instance_variable_means_the_default_instance() {
    // Act
    let instance = Instance::from_env(env(&[HOME, APPDATA, (INSTANCE_VAR, "")])).unwrap();

    // Assert
    assert!(instance.is_default());
}

#[test]
fn a_named_instance_shares_no_files_with_the_default_one() {
    // Arrange
    let default = Instance::from_env(env(&[HOME, APPDATA])).unwrap();

    // Act
    let named = Instance::from_env(env(&[HOME, APPDATA, (INSTANCE_VAR, "test-1")])).unwrap();

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
    // Act
    let instance = Instance::from_env(env(&[
        (INSTANCE_VAR, "ci"),
        (CONFIG_DIR_VAR, r"D:\tmp\cfg"),
        (DATA_DIR_VAR, r"D:\tmp\data"),
    ]))
    .unwrap();

    // Assert -- no home or app data directory is needed at all.
    assert_eq!(instance.config_dir(), Path::new(r"D:\tmp\cfg"));
    assert_eq!(
        instance.pid_path(),
        Path::new(r"D:\tmp\data").join("mosaico.pid")
    );
}

#[test]
fn a_bad_name_is_refused_rather_than_ignored() {
    // Act / Assert -- falling back to the default would reach the
    // user's own daemon.
    for bad in ["../escape", "has space", "pipe\\name", &"x".repeat(65)] {
        let result = Instance::from_env(env(&[HOME, APPDATA, (INSTANCE_VAR, bad)]));
        assert!(
            matches!(result, Err(InstanceError::InvalidName(_))),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn a_missing_home_directory_is_reported() {
    // Act
    let result = Instance::from_env(env(&[APPDATA]));

    // Assert
    assert_eq!(result, Err(InstanceError::NoDirectory("config")));
}

#[test]
fn explicit_instances_validate_their_name() {
    // Act / Assert
    assert!(Instance::new(Some("ok_name-2"), "c", "d").is_ok());
    assert!(Instance::new(Some(""), "c", "d").is_err());
    assert!(Instance::new(None, "c", "d").unwrap().is_default());
}
