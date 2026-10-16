//! What doctor prints about transitions, from `config.toml` and the fake
//! desktop.

use super::super::*;
use super::SNAPS;
use crate::commands::context::Exit;
use crate::commands::test_support::{Frames, Harness};

/// What `check_animations` prints with `config.toml` holding `contents`,
/// or with no file when it is empty, on the desktop `setup` leaves.
fn checked(test: &str, contents: &str, setup: impl FnOnce(&Harness)) -> Vec<String> {
    let mut h = Harness::new(test);
    if !contents.is_empty() {
        std::fs::create_dir_all(h.config.dir()).unwrap();
        std::fs::write(h.config.config_path(), contents).unwrap();
    }
    setup(&h);
    h.run(|cx| {
        check_animations(cx);
        Exit::Success
    });
    let _ = std::fs::remove_dir_all(h.config.dir());
    h.out().lines().map(str::to_string).collect()
}

#[test]
fn transitions_are_reported_off_by_default_with_every_setting() {
    // Arrange / Act -- no config.toml, on a desktop with frames.
    let lines = checked("doctor-animations-default", "", |h| {
        h.world.frames.set(Frames::Supported);
    });

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Window transitions are off ([animations] enabled = false)"),
            "       enabled = false, duration = 180 ms, easing = ease-out-cubic, \
             follow_system = true"
                .to_string(),
            "       The system allows animations".to_string(),
            "       This desktop moves windows frame by frame".to_string(),
        ]
    );
}

#[test]
fn the_settings_the_system_and_the_desktop_are_each_reported() {
    // Arrange -- on in the file, while the system turns animations off,
    // on a desktop without frames.
    let contents = "[animations]\nenabled = true\nduration = 250\neasing = \"linear\"\n";

    // Act
    let lines = checked("doctor-animations-file", contents, |h| {
        h.world.animations_allowed.set(false);
    });

    // Assert
    let why = "the system turns animations off (follow_system = true)";
    assert_eq!(lines[0], format!("  {WARN} {SNAPS} {why}"));
    assert_eq!(
        lines[1..],
        [
            "       enabled = true, duration = 250 ms, easing = linear, follow_system = true",
            "       The system turns animations off (its setting is off, or this is a remote session)",
            "       This desktop cannot move windows frame by frame",
        ]
    );
}

#[test]
fn enabled_transitions_on_a_desktop_that_cannot_be_asked_are_neither_on_nor_snapping() {
    // Arrange / Act
    let lines = checked(
        "doctor-animations-fails",
        "[animations]\nenabled = true\n",
        |h| {
            h.world.frames.set(Frames::Fails);
        },
    );

    // Assert -- the failure is flagged, and gives no reason to snap.
    let unknown = "Window transitions are on, but whether windows can move frame by frame \
                   is unknown";
    assert_eq!(lines[0], format!("  {WARN} {unknown}"));
    let failure = format!("  {FAIL} Could not ask whether windows can move frame by frame: ");
    assert!(lines[3].starts_with(&failure), "{lines:?}");
}

#[test]
fn an_invalid_config_is_reported_with_the_defaults_named_as_such() {
    // Arrange -- an easing mosaico does not take makes the file invalid.
    let contents = "[animations]\nenabled = true\neasing = \"bounce\"\n";

    // Act
    let lines = checked("doctor-animations-invalid", contents, |_| {});

    // Assert -- not as the file's own values.
    let off = format!("  {OK} Window transitions are off (the default, as config.toml is invalid)");
    assert_eq!(lines[0], off);
    let defaults = "       The defaults, as config.toml is invalid: enabled = false,";
    assert!(lines[1].starts_with(defaults), "{lines:?}");
}
