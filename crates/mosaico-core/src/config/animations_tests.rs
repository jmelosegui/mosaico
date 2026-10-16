use std::time::Duration;

use crate::config::Config;
use crate::config::store::test_support::temp_store;
use crate::motion::{CubicBezier, Easing};

use super::*;

#[test]
fn transitions_are_on_by_default_and_last_180_ms_easing_out() {
    // Act
    let mut config = Config::default();
    config.validate();

    // Assert
    let animations = &config.animations;
    assert!(animations.enabled);
    assert_eq!(animations.duration, 180);
    assert_eq!(animations.easing, Easing::EaseOutCubic);
    assert!(animations.follow_system);
}

#[test]
fn a_config_without_the_section_has_the_defaults() {
    // Act
    let config: Config = toml::from_str("[layout]\ngap = 12\n").unwrap();

    // Assert
    assert_eq!(config.animations, AnimationConfig::default());
}

#[test]
fn the_section_is_read_from_config_toml() {
    // Arrange
    let toml = "[animations]\nenabled = true\nduration = 250\n\
                easing = \"cubic-bezier(0.2, 0.0, 0.0, 1.0)\"\nfollow_system = false\n";

    // Act
    let config: Config = toml::from_str(toml).unwrap();

    // Assert
    let curve = CubicBezier::new(0.2, 0.0, 0.0, 1.0).unwrap();
    let expected = AnimationConfig {
        enabled: true,
        duration: 250,
        easing: Easing::CubicBezier(curve),
        follow_system: false,
    };
    assert_eq!(config.animations, expected);
}

#[test]
fn validate_clamps_the_duration_to_at_most_a_second() {
    for (written, kept) in [(-5, 0), (0, 0), (500, 500), (1000, 1000), (5000, 1000)] {
        // Arrange
        let mut config = Config {
            animations: AnimationConfig {
                duration: written,
                ..Default::default()
            },
            ..Default::default()
        };

        // Act
        config.validate();

        // Assert
        assert_eq!(config.animations.duration, kept, "duration = {written}");
    }
}

#[test]
fn a_transition_follows_the_configured_duration_and_easing() {
    // Arrange -- and one whose duration validation never clamped.
    let config = AnimationConfig {
        duration: 250,
        easing: Easing::Linear,
        ..Default::default()
    };
    let unchecked = AnimationConfig {
        duration: 99_999,
        ..Default::default()
    };

    // Act
    let (spec, long) = (config.spec(), unchecked.spec());

    // Assert
    assert_eq!(spec.duration, Duration::from_millis(250));
    assert_eq!(spec.easing, Easing::Linear);
    assert_eq!(long.duration, Duration::from_secs(1));
}

#[test]
fn an_unknown_easing_makes_config_toml_invalid_as_other_unknown_values_do() {
    // Arrange -- as `hiding = "fade"` would: the daemon starts with the
    // defaults, doctor reports the file, and a reload keeps the config in
    // use.
    let store = temp_store("animations-unknown-easing");
    let file = "[layout]\ngap = 12\n\n[animations]\neasing = \"bouncy\"\n";
    std::fs::write(store.config_path(), file).unwrap();

    // Act
    let error = store.try_load_config().unwrap_err().to_string();
    let loaded = store.load_config();

    // Assert -- the message names the file and the easing.
    let file_error = format!("{}: TOML parse error", store.config_path().display());
    assert!(error.starts_with(&file_error), "{error}");
    assert!(error.contains("unknown easing \"bouncy\""), "{error}");
    assert_eq!(loaded.layout.gap, Config::default().layout.gap);
}

#[test]
fn a_cubic_bezier_that_overshoots_makes_config_toml_invalid() {
    // Arrange -- CSS takes it, but it would carry windows past their
    // targets.
    let toml = "[animations]\neasing = \"cubic-bezier(0.68, -0.6, 0.32, 1.6)\"\n";

    // Act
    let error = toml::from_str::<Config>(toml).unwrap_err().to_string();

    // Assert
    assert!(
        error.contains("all four numbers must be within 0 to 1"),
        "{error}"
    );
}

#[test]
fn transitions_that_take_no_time_count_as_off() {
    // Arrange
    let mut animations = AnimationConfig {
        enabled: true,
        ..Default::default()
    };

    // Act / Assert -- and below zero, where validation has not clamped.
    assert!(animations.is_on());
    for duration in [0, -5] {
        animations.duration = duration;
        assert!(!animations.is_on(), "duration = {duration}");
    }
    assert!(AnimationConfig::default().is_on());
}
