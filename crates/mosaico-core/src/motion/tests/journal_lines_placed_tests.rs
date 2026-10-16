use crate::id::wid;
use crate::motion::test_support::ms;

use super::*;

#[test]
fn a_kept_line_says_when_the_window_went_to_the_target_it_kept_and_how() {
    // Arrange
    let over = KeptLine {
        window: wid(0x1A),
        when: Kept::ChangeOver,
        placed: Placed::AtOnce,
    };
    let landing = KeptLine {
        placed: Placed::Landing,
        ..over
    };
    let stopping = KeptLine {
        when: Kept::Shutdown,
        placed: Placed::PostedThere,
        ..over
    };

    // Act / Assert
    assert_eq!(
        over.to_string(),
        "transition: 0x1A placed at the target it kept, the display change over"
    );
    assert_eq!(
        landing.to_string(),
        "transition: 0x1A placed at the target it kept, the display change over; its \
         placement posted behind what was on its way"
    );
    assert_eq!(
        stopping.to_string(),
        "transition: 0x1A placed at the target it kept as the daemon stopped; its placement \
         posted behind what was on its way, the window there already"
    );
}

#[test]
fn a_landed_line_says_how_the_window_was_found_to_land_and_how_long_it_took() {
    // Arrange
    let there = LandedLine {
        window: wid(0x2B),
        after: ms(36),
        how: Landed::There,
    };
    let away = LandedLine {
        after: ms(120),
        how: Landed::Away,
        ..there
    };
    let overdue = LandedLine {
        after: ms(516),
        how: Landed::Overdue,
        ..there
    };
    let late = LandedLine {
        after: ms(2_310),
        how: Landed::Late,
        ..there
    };

    // Act / Assert
    assert_eq!(
        there.to_string(),
        "transition: 0x2B is in place, 36 ms after its placement was posted"
    );
    assert_eq!(
        away.to_string(),
        "transition: 0x2B is not where it was placed, with nothing on its way, 120 ms after its \
         placement was posted"
    );
    assert_eq!(
        overdue.to_string(),
        "transition: 0x2B is taken to be in place, not reported there 516 ms after its \
         placement was posted"
    );
    assert_eq!(
        late.to_string(),
        "transition: 0x2B is taken to be in place as it moves at last, 2310 ms after its \
         placement was posted"
    );
}

#[test]
fn a_window_that_reports_where_it_is_lands_there_late_or_away() {
    // Act / Assert
    assert_eq!(Landed::reported(true, false), Landed::There);
    assert_eq!(Landed::reported(true, true), Landed::There);
    assert_eq!(Landed::reported(false, true), Landed::Late);
    assert_eq!(Landed::reported(false, false), Landed::Away);
}
