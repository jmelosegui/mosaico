//! The config files that a merge or a migration rewrites keep their line
//! endings and their byte order mark.

use std::path::Path;

use crate::config::store::merge_unfocused::append_missing_unfocused_border_hint;
use crate::config::store::merge_wrap::add_missing_wrap_key;
use crate::config::store::migrate::migrate_config_if_needed;
use crate::config::store::test_support::{cleanup, unique_temp_path};

use super::BOM;

/// What a file that holds `original` holds once `change` ran on it.
fn changed(original: &str, change: impl FnOnce(&Path, &str)) -> String {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    change(&path, original);
    let after = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    after
}

fn wrap(path: &Path, _: &str) {
    assert!(add_missing_wrap_key(path).unwrap());
}

fn unfocused(path: &Path, _: &str) {
    assert!(append_missing_unfocused_border_hint(path).unwrap());
}

fn migrate(path: &Path, original: &str) {
    let migrated = migrate_config_if_needed(path, original).unwrap();
    assert_eq!(migrated, std::fs::read_to_string(path).ok());
}

/// `text` with every line ending in CRLF.
fn crlf(text: &str) -> String {
    text.replace('\n', "\r\n")
}

/// Checks that `change` writes back a CRLF file of `lf` in CRLF, and one
/// that starts with a byte order mark with it, as it writes `lf` itself.
fn keeps_the_formatting(lf: &str, change: fn(&Path, &str)) {
    let expected = changed(lf, change);
    assert_eq!(changed(&crlf(lf), change), crlf(&expected));
    assert_eq!(
        changed(&format!("{BOM}{lf}"), change),
        format!("{BOM}{expected}")
    );
}

#[test]
fn the_wrap_key_keeps_the_line_endings_and_the_byte_order_mark() {
    // Arrange
    let lf = "[workspaces]\nmode = \"global\"\n\n[borders]\nwidth = 2\n";

    // Act / Assert
    keeps_the_formatting(lf, wrap);
}

#[test]
fn the_unfocused_color_keeps_the_line_endings_and_the_byte_order_mark() {
    // Arrange
    let lf = "[borders.colors]\nfocused = \"blue\"\n";

    // Act / Assert
    keeps_the_formatting(lf, unfocused);
}

#[test]
fn a_migration_keeps_the_line_endings_and_the_byte_order_mark() {
    // Arrange
    let lf = "[layout]\ngap = 8\n\n[layout.workspaces]\n1 = \"vertical-stack\"\n";

    // Act / Assert
    keeps_the_formatting(lf, migrate);
}
