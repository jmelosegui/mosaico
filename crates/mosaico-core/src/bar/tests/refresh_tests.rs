use std::time::Duration;

use super::*;

/// A bar with `widgets`, inline tables of `bar.toml`, on its right.
fn right(widgets: &str) -> BarConfig {
    toml::from_str(&format!("left = []\nright = [{widgets}]")).expect("a valid bar.toml")
}

/// The time at `second` past some minute.
fn at(second: u8) -> LocalTime {
    LocalTime {
        second,
        ..LocalTime::default()
    }
}

#[test]
fn a_bar_with_nothing_that_changes_on_its_own_never_draws_again_by_itself() {
    // Arrange
    let config = right(r#"{ type = "workspaces" }, { type = "layout" }"#);

    // Act
    let due = refresh_in(&config, at(10));

    // Assert
    assert_eq!(due, None);
}

#[test]
fn a_clock_without_seconds_draws_again_as_the_minute_turns() {
    // Arrange
    let config = right(r#"{ type = "clock", format = "%H:%M" }"#);

    // Act
    let due = refresh_in(&config, at(15));

    // Assert
    assert_eq!(due, Some(Duration::from_secs(45)));
}

#[test]
fn a_clock_with_seconds_draws_again_every_second() {
    // Arrange
    let config = right(r#"{ type = "clock", format = "%H:%M:%S" }"#);

    // Act
    let due = refresh_in(&config, at(15));

    // Assert
    assert_eq!(due, Some(READING_INTERVAL));
}

#[test]
fn a_literal_percent_before_an_s_is_not_seconds() {
    // Arrange
    let config = right(r#"{ type = "date", format = "%d 100%%S" }"#);

    // Act
    let due = refresh_in(&config, at(0));

    // Assert
    assert_eq!(due, Some(Duration::from_secs(60)));
}

#[test]
fn a_reading_of_the_machine_is_taken_again_every_second() {
    // Arrange
    let config = right(r#"{ type = "clock", format = "%H:%M" }, { type = "media" }"#);

    // Act
    let due = refresh_in(&config, at(15));

    // Assert
    assert_eq!(due, Some(READING_INTERVAL));
}

#[test]
fn a_disabled_widget_draws_nothing_again() {
    // Arrange
    let config = right(r#"{ type = "cpu", enabled = false }, { type = "date", format = "%d" }"#);

    // Act
    let due = refresh_in(&config, at(30));

    // Assert
    assert_eq!(due, Some(Duration::from_secs(30)));
}
