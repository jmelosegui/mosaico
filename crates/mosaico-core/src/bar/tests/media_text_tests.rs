//! The media widget's text, from what a fake machine is playing.

use super::tests::{label_of, shows};
use crate::bar::fakes::FakeSystem;
use crate::bar::{BarState, NowPlaying};

/// A machine playing `title` by `artist`.
fn playing(artist: &str, title: &str) -> FakeSystem {
    let system = FakeSystem::default();
    let track = NowPlaying {
        artist: artist.into(),
        title: title.into(),
    };
    system.set(|m| m.playing = Some(track));
    system
}

/// What a media widget without an icon reads on `system`.
fn media_label(max_length: usize, system: &FakeSystem) -> String {
    let widget = format!(r#"{{ type = "media", icon = "", max_length = {max_length} }}"#);
    label_of(&widget, &BarState::default(), system)
}

#[test]
fn the_media_widget_shows_the_artist_and_the_title() {
    // Act
    let label = media_label(40, &playing("Artist", "Title"));

    // Assert
    assert_eq!(label, "Artist - Title");
}

#[test]
fn the_media_widget_shows_only_the_title_without_an_artist() {
    // Act
    let label = media_label(40, &playing("", "Title"));

    // Assert
    assert_eq!(label, "Title");
}

#[test]
fn the_media_widget_shows_only_the_artist_without_a_title() {
    // Act
    let label = media_label(40, &playing("Artist", ""));

    // Assert
    assert_eq!(label, "Artist");
}

#[test]
fn the_media_widget_shows_its_icon_before_the_track() {
    // Act
    let label = label_of(
        r#"{ type = "media" }"#,
        &BarState::default(),
        &playing("Artist", "Title"),
    );

    // Assert
    assert_eq!(label, "\u{F001} Artist - Title");
}

#[test]
fn the_media_widget_hides_when_nothing_plays() {
    // Act
    let shown = shows(
        r#"{ type = "media" }"#,
        &BarState::default(),
        &FakeSystem::default(),
    );

    // Assert
    assert!(!shown);
}

#[test]
fn the_media_widget_hides_when_media_cannot_be_read() {
    // Arrange
    let system = playing("Artist", "Title");
    system.set(|m| m.media_fails = true);

    // Act
    let shown = shows(r#"{ type = "media" }"#, &BarState::default(), &system);

    // Assert
    assert!(!shown);
}

#[test]
fn a_long_track_is_cut_to_the_maximum_length() {
    // Act
    let label = media_label(
        20,
        &playing("Very Long Artist Name", "Very Long Track Title"),
    );

    // Assert
    assert_eq!(label, "Very Long Artist ...");
}

#[test]
fn a_track_that_fits_exactly_is_not_cut() {
    // Act
    let label = media_label(14, &playing("Artist", "Title"));

    // Assert
    assert_eq!(label, "Artist - Title");
}

#[test]
fn a_maximum_length_of_zero_leaves_only_the_icon() {
    // Arrange
    let system = playing("Artist", "Title");

    // Act
    let shown = shows(
        r#"{ type = "media", max_length = 0 }"#,
        &BarState::default(),
        &system,
    );
    let label = label_of(
        r#"{ type = "media", max_length = 0 }"#,
        &BarState::default(),
        &system,
    );

    // Assert
    assert!(shown);
    assert_eq!(label, "\u{F001}");
}

#[test]
fn a_maximum_length_under_three_still_ends_in_dots() {
    // Act
    let label = media_label(2, &playing("Artist", "Title"));

    // Assert
    assert_eq!(label, "...");
}
