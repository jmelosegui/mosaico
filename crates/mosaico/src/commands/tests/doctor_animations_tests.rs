use mosaico_core::Error;
use mosaico_core::config::AnimationConfig;

use super::*;

#[path = "doctor_animations_check_tests.rs"]
mod check_tests;

const SNAPS: &str = "Window transitions are on, but windows snap into place:";

/// `animations` as read from `config.toml`.
fn from_file(animations: AnimationConfig) -> Settings {
    Settings {
        animations,
        defaults_for_invalid_file: false,
    }
}

fn enabled() -> AnimationConfig {
    AnimationConfig {
        enabled: true,
        ..AnimationConfig::default()
    }
}

fn unsupported() -> Result<()> {
    Err(Error::Unsupported("moving windows frame by frame"))
}

#[test]
fn windows_snap_for_the_first_reason_that_holds() {
    // Arrange
    let instant = AnimationConfig {
        duration: 0,
        ..enabled()
    };
    let not_following = AnimationConfig {
        follow_system: false,
        ..enabled()
    };
    let reasons = [
        "they take no time (duration = 0)",
        "the system turns animations off (follow_system = true)",
        "this desktop cannot move windows frame by frame",
    ];

    // Act / Assert -- with the system against them, on a desktop without
    // frames.
    for (settings, why) in [instant, enabled(), not_following].into_iter().zip(reasons) {
        let line = status_line(&from_file(settings.clone()), false, &unsupported());
        assert_eq!(line, format!("  {WARN} {SNAPS} {why}"), "{settings:?}");
    }
}

#[test]
fn transitions_are_on_when_nothing_stops_them() {
    // Arrange -- the near miss: a system against them that is not followed.
    let not_following = AnimationConfig {
        follow_system: false,
        ..enabled()
    };

    // Act / Assert
    let on = format!("  {OK} Window transitions are on");
    assert_eq!(status_line(&from_file(enabled()), true, &Ok(())), on);
    assert_eq!(status_line(&from_file(not_following), false, &Ok(())), on);
}

#[test]
fn a_desktop_that_cannot_be_asked_about_frames_gives_no_reason_to_snap() {
    // Arrange -- any failure other than a platform without frames.
    let failed = Err(Error::platform(
        "move windows frame by frame",
        "access denied",
    ));

    // Act
    let line = status_line(&from_file(enabled()), true, &failed);

    // Assert
    let unknown = "Window transitions are on, but whether windows can move frame by frame \
                   is unknown";
    assert_eq!(line, format!("  {WARN} {unknown}"));
}
