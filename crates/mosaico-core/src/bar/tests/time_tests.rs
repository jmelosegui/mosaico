use super::*;

/// Tuesday 13 October 2026, 09:05:07.
fn tuesday_morning() -> LocalTime {
    LocalTime {
        year: 2026,
        month: 10,
        day: 13,
        weekday: 2,
        hour: 9,
        minute: 5,
        second: 7,
    }
}

#[test]
fn hours_minutes_and_seconds_are_padded_to_two_digits() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("%H:%M:%S");

    // Assert
    assert_eq!(text, "09:05:07");
}

#[test]
fn the_date_spells_out_the_weekday_and_the_month() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("%A %d %B %Y");

    // Assert
    assert_eq!(text, "Tuesday 13 October 2026");
}

#[test]
fn the_day_is_padded_to_two_digits() {
    // Arrange
    let time = LocalTime {
        day: 3,
        ..tuesday_morning()
    };

    // Act
    let text = time.format("%d");

    // Assert
    assert_eq!(text, "03");
}

#[test]
fn text_around_the_specifiers_is_kept() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("at %H h");

    // Assert
    assert_eq!(text, "at 09 h");
}

#[test]
fn a_double_percent_sign_is_a_literal_percent_sign() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("100%%");

    // Assert
    assert_eq!(text, "100%");
}

#[test]
fn an_unknown_specifier_is_kept_as_written() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("%H%Q");

    // Assert
    assert_eq!(text, "09%Q");
}

#[test]
fn a_trailing_percent_sign_is_kept() {
    // Arrange
    let time = tuesday_morning();

    // Act
    let text = time.format("%H%");

    // Assert
    assert_eq!(text, "09%");
}

#[test]
fn a_weekday_or_month_out_of_range_shows_question_marks() {
    // Arrange
    let after = LocalTime {
        weekday: 7,
        month: 13,
        ..tuesday_morning()
    };
    let before = LocalTime {
        month: 0,
        ..tuesday_morning()
    };

    // Act
    let after = after.format("%A %B");
    let before = before.format("%B");

    // Assert
    assert_eq!(after, "??? ???");
    assert_eq!(before, "???");
}
