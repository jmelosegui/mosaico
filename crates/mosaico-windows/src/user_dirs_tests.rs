use std::collections::HashMap;
use std::path::Path;

use mosaico_core::Instance;
use mosaico_core::instance::{INSTANCE_VAR, InstanceError};

use super::*;

/// Builds an environment lookup from key/value pairs.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let map: HashMap<String, OsString> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |key| map.get(key).cloned()
}

/// The instance the CLI finds at startup in an environment of `pairs`.
fn instance(pairs: &[(&str, &str)]) -> Result<Instance, InstanceError> {
    Instance::from_env(env(pairs))
}

const PROFILE: (&str, &str) = ("USERPROFILE", r"C:\Users\me");
const HOME: (&str, &str) = ("HOME", r"D:\home\me");
const LOCAL_APP_DATA: (&str, &str) = ("LOCALAPPDATA", r"C:\Users\me\AppData\Local");

#[test]
fn the_config_folder_is_dot_config_in_the_profile_folder() {
    // Act
    let dirs = WindowsUserDirs::from_env(env(&[PROFILE, HOME]));

    // Assert
    assert_eq!(
        dirs.config_dir(),
        UserDir {
            setting: "USERPROFILE",
            value: Some(PathBuf::from(r"C:\Users\me")),
            subfolder: Some(".config"),
        }
    );
}

#[test]
fn home_stands_in_only_for_a_profile_folder_that_is_not_set() {
    // Act
    let unset = WindowsUserDirs::from_env(env(&[HOME])).config_dir();
    let empty = WindowsUserDirs::from_env(env(&[("USERPROFILE", ""), HOME])).config_dir();

    // Assert
    assert_eq!(
        (unset.setting, unset.value),
        ("HOME", Some(PathBuf::from(r"D:\home\me")))
    );
    assert_eq!(
        (empty.setting, empty.value),
        ("USERPROFILE", Some(PathBuf::new()))
    );
}

#[test]
fn the_data_folder_is_the_local_app_data_folder() {
    // Act
    let dirs = WindowsUserDirs::from_env(env(&[LOCAL_APP_DATA]));

    // Assert
    assert_eq!(
        dirs.data_dir(),
        UserDir {
            setting: "LOCALAPPDATA",
            value: Some(PathBuf::from(r"C:\Users\me\AppData\Local")),
            subfolder: None,
        }
    );
}

#[test]
fn variables_are_reported_as_found() {
    // Act -- whether a value can be used is for the instance to say.
    let set = WindowsUserDirs::from_env(env(&[("USERPROFILE", r"Users\me"), ("LOCALAPPDATA", "")]));
    let unset = WindowsUserDirs::from_env(env(&[]));

    // Assert
    assert_eq!(set.config_dir().value, Some(PathBuf::from(r"Users\me")));
    assert_eq!(set.data_dir().value, Some(PathBuf::new()));
    assert_eq!(
        (unset.config_dir().setting, unset.config_dir().value),
        ("USERPROFILE", None)
    );
    assert_eq!(
        (unset.data_dir().setting, unset.data_dir().value),
        ("LOCALAPPDATA", None)
    );
}

#[test]
fn the_default_instance_finds_its_files_where_it_always_has() {
    // Act
    let instance = instance(&[PROFILE, HOME, LOCAL_APP_DATA]).unwrap();

    // Assert -- the folders every existing install uses.
    assert_eq!(
        instance.config_dir(),
        Path::new(r"C:\Users\me\.config\mosaico")
    );
    assert_eq!(
        instance.log_dir(),
        Path::new(r"C:\Users\me\.config\mosaico\logs")
    );
    assert_eq!(
        instance.data_dir(),
        Path::new(r"C:\Users\me\AppData\Local\mosaico")
    );
    assert_eq!(
        instance.pid_path(),
        Path::new(r"C:\Users\me\AppData\Local\mosaico\mosaico.pid")
    );
}

#[test]
fn a_named_instance_finds_its_files_under_instances() {
    // Act
    let instance = instance(&[PROFILE, LOCAL_APP_DATA, (INSTANCE_VAR, "ci")]).unwrap();

    // Assert
    assert_eq!(
        instance.config_dir(),
        Path::new(r"C:\Users\me\.config\mosaico\instances\ci")
    );
    assert_eq!(
        instance.pid_path(),
        Path::new(r"C:\Users\me\AppData\Local\mosaico\instances\ci\mosaico.pid")
    );
}

#[test]
fn without_a_profile_folder_the_config_is_found_in_home() {
    // Act
    let instance = instance(&[HOME, LOCAL_APP_DATA]).unwrap();

    // Assert
    assert_eq!(
        instance.config_dir(),
        Path::new(r"D:\home\me\.config\mosaico")
    );
}

#[test]
fn a_missing_folder_is_refused() {
    // Act / Assert
    assert_eq!(
        instance(&[LOCAL_APP_DATA]),
        Err(InstanceError::NoDirectory("config"))
    );
    assert_eq!(
        instance(&[PROFILE]),
        Err(InstanceError::NoDirectory("data"))
    );
}
