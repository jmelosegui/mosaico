//! The examples and schemas under `docs/bar-api` are what bar authors read,
//! so they are checked against the types: each example reads and writes
//! back byte for byte, and each schema is the one the types make.
//!
//! After changing the types on purpose, run these tests with
//! `MOSAICO_BLESS_API=1` to write the files again, and review the diff.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{State, Update};

pub(super) fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/bar-api")
        .join(name)
}

pub(super) fn pretty(value: &impl Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default() + "\n"
}

/// Compares `expected` to the file `name`, or writes it there when
/// blessing.
pub(super) fn check(name: &str, expected: &str) {
    let path = path(name);
    if std::env::var_os("MOSAICO_BLESS_API").is_some() {
        let _ = std::fs::write(&path, expected);
        return;
    }
    let found = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert_eq!(
        found, expected,
        "{name} is out of date; see the module docs"
    );
}

fn round_trip<T: Serialize + DeserializeOwned>(name: &str) {
    // Arrange
    let text = std::fs::read_to_string(path(name))
        .unwrap_or_default()
        .replace("\r\n", "\n");

    // Act
    let value: Result<T, _> = serde_json::from_str(&text);

    // Assert
    let value = value.unwrap_or_else(|e| panic!("{name} does not read: {e}"));
    assert_eq!(pretty(&value), text, "{name} does not write back the same");
}

#[test]
fn the_state_example_reads_and_writes_back_the_same() {
    round_trip::<State>("state.json");
}

#[test]
fn the_update_example_reads_and_writes_back_the_same() {
    round_trip::<Update>("update.json");
}

#[test]
fn the_state_schema_is_up_to_date() {
    check("state.schema.json", &pretty(&schemars::schema_for!(State)));
}

#[test]
fn the_update_schema_is_up_to_date() {
    check(
        "update.schema.json",
        &pretty(&schemars::schema_for!(Update)),
    );
}

#[test]
fn a_bar_reads_a_state_with_fields_it_does_not_know_and_misses_some() {
    // Arrange
    let text = r#"{"daemon": {"version": "9.0.0", "future": true}, "later": [1]}"#;

    // Act
    let state: Result<State, _> = serde_json::from_str(text);

    // Assert
    let state = state.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state.daemon.version, "9.0.0");
    assert!(state.monitors.is_empty());
}
