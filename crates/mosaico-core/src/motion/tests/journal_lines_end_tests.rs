use crate::id::wid;
use crate::motion::test_support::ms;

use super::*;

/// The line of window 0x1A's motion, ended at its target, with `pending`
/// of its 12 frames found pending.
fn ended(pending: u32) -> EndLine {
    EndLine {
        window: wid(0x1A),
        ending: Ending::AtTarget,
        took: ms(192),
        planned: ms(180),
        sent: 12,
        handled: 12 - pending,
        pending,
        placed: None,
    }
}

#[test]
fn a_behind_line_says_at_which_frame_the_window_fell_behind() {
    // Arrange
    let line = BehindLine {
        window: wid(0x2B),
        frame: 7,
    };

    // Act / Assert
    assert_eq!(
        line.to_string(),
        "transition: 0x2B fell behind at its frame 7: its step goes straight to its targets"
    );
}

#[test]
fn an_end_line_gives_the_time_against_the_duration_and_the_frames() {
    // Act
    let texts = [ended(2), ended(0)].map(|line| line.to_string());

    // Assert
    assert_eq!(
        texts,
        [
            "transition: 0x1A ended at its target after 192 ms of 180: 12 frame(s) sent, \
             10 handled, 2 found pending",
            "transition: 0x1A ended at its target after 192 ms of 180: 12 frame(s) sent, \
             12 handled, none found pending",
        ]
    );
}

#[test]
fn a_step_line_at_its_end_gives_its_windows_its_frames_and_the_slowest() {
    // Arrange
    let done = StepDoneLine {
        windows: vec![wid(0x1A), wid(0x2B)],
        sent: 24,
        handled: 21,
        pending: 3,
        slowest: Some((wid(0x2B), 3)),
    };
    let smooth = StepDoneLine {
        sent: 24,
        handled: 24,
        pending: 0,
        slowest: None,
        ..done.clone()
    };

    // Act / Assert
    assert_eq!(
        done.to_string(),
        "transition: a step of 2 window(s) is done: 0x1A, 0x2B; 24 frame(s) sent, 21 \
         handled, 3 found pending; slowest 0x2B, 3 pending"
    );
    assert_eq!(
        smooth.to_string(),
        "transition: a step of 2 window(s) is done: 0x1A, 0x2B; 24 frame(s) sent, 24 \
         handled, none found pending"
    );
}

#[test]
fn an_ending_line_reads_as_the_line_it_holds() {
    // Arrange
    let motion = ended(1);
    let step = StepDoneLine {
        windows: vec![wid(0x1A)],
        sent: 12,
        handled: 11,
        pending: 1,
        slowest: Some((wid(0x1A), 1)),
    };

    // Act / Assert
    assert_eq!(EndingLine::Motion(motion).to_string(), motion.to_string());
    assert_eq!(EndingLine::Step(step.clone()).to_string(), step.to_string());
}

#[test]
fn an_end_line_says_how_the_placement_that_ended_it_was_made() {
    // Arrange
    let placed = |placed| EndLine {
        placed: Some(placed),
        ..ended(0)
    };
    let line = "transition: 0x1A ended at its target after 192 ms of 180: 12 frame(s) sent, 12 \
                handled, none found pending";
    let posted = "; its placement posted behind what was on its way";

    // Act / Assert
    assert_eq!(placed(Placed::AtOnce).to_string(), line);
    assert_eq!(
        placed(Placed::PostedThere).to_string(),
        format!("{line}{posted}, the window there already")
    );
    assert_eq!(
        placed(Placed::Landing).to_string(),
        format!("{line}{posted}")
    );
}
