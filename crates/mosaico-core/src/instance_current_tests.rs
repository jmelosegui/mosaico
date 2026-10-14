use std::collections::HashMap;

use super::*;
use crate::instance::INSTANCE_VAR;
use crate::user_dirs::UserDir;

/// Builds an environment lookup from key/value pairs.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let map: HashMap<String, OsString> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |key| map.get(key).cloned()
}

/// A platform that keeps both directories in the temporary folder.
struct Temp;

impl UserDirs for Temp {
    fn config_dir(&self) -> UserDir {
        UserDir {
            setting: "TEMP",
            value: Some(std::env::temp_dir()),
            subfolder: None,
        }
    }

    fn data_dir(&self) -> UserDir {
        self.config_dir()
    }
}

#[test]
fn nothing_is_current_before_an_instance_is_resolved() {
    // Arrange
    let slot = Slot::new();

    // Act / Assert
    assert_eq!(slot.get(), None);
}

#[test]
fn the_first_instance_resolved_stays_current() {
    // Arrange
    let slot = Slot::new();
    let first = slot
        .init_from(env(&[(INSTANCE_VAR, "first")]), &Temp)
        .unwrap();

    // Act
    let again = slot
        .init_from(env(&[(INSTANCE_VAR, "second")]), &Temp)
        .unwrap();

    // Assert
    assert!(std::ptr::eq(first, again));
    assert_eq!(slot.get().and_then(Instance::name), Some("first"));
}

#[test]
fn a_failed_resolution_is_not_kept() {
    // Arrange
    let slot = Slot::new();
    let failed = slot.init_from(env(&[(INSTANCE_VAR, "not valid")]), &Temp);

    // Act
    let resolved = slot.init_from(env(&[]), &Temp);

    // Assert
    assert!(matches!(failed, Err(InstanceError::InvalidName(_))));
    assert!(resolved.is_ok_and(Instance::is_default));
    assert!(slot.get().is_some());
}

#[test]
fn a_test_process_has_no_instance() {
    // Act / Assert -- no unit test resolves the instance of the process,
    // so none of them reaches the user's real folders.
    assert_eq!(Instance::current(), None);
}

#[test]
fn only_the_instance_resolved_is_current() {
    // Arrange
    let slot = Slot::new();
    let other = Instance::new(Some("other"), "cfg", "data").unwrap();
    let before = slot.require(&other);
    let resolved = slot.init_from(env(&[]), &Temp).unwrap().clone();

    // Act / Assert -- the daemon refuses to run on an instance whose
    // logs and record of hidden windows it would not find.
    assert_eq!(before, Err(InstanceError::NotCurrent));
    assert_eq!(slot.require(&resolved), Ok(()));
    assert_eq!(slot.require(&other), Err(InstanceError::NotCurrent));
}
