use crate::Rect;

use super::super::{ApplyFacts, StepFacts, step_animates, still_animates};
use super::*;

/// A placement of a step that passes every rule, changed by `change`.
fn step(change: impl FnOnce(&mut StepFacts)) -> StepFacts {
    let mut facts = StepFacts {
        enabled: true,
        appears: false,
        behind_monocle: false,
        rebuild: false,
        same_monitor: true,
        windows_moved: 2,
    };
    change(&mut facts);
    facts
}

/// A placement that passes every rule as it is applied, a window moving
/// across half of a 1600 px screen, changed by `change`.
fn applied(change: impl FnOnce(&mut ApplyFacts)) -> ApplyFacts {
    let mut facts = ApplyFacts {
        follow_system: true,
        system_allows: true,
        minimized: false,
        maximized: false,
        frames_supported: true,
        from: Rect::new(0, 0, 800, 900),
        to: Rect::new(800, 0, 800, 900),
    };
    change(&mut facts);
    facts
}

#[test]
fn a_placement_that_passes_every_rule_has_no_reason_to_snap() {
    // Act / Assert
    assert_eq!(step_snap(&step(|_| {})), None);
    assert_eq!(apply_snap(&applied(|_| {})), None);
}

#[test]
fn each_rule_of_the_step_decision_gives_its_own_reason() {
    // Arrange
    let cases = [
        (step(|f| f.enabled = false), StepSnap::Off),
        (step(|f| f.rebuild = true), StepSnap::Rebuild),
        (step(|f| f.windows_moved = 17), StepSnap::TooManyWindows(17)),
        (step(|f| f.appears = true), StepSnap::Appears),
        (step(|f| f.behind_monocle = true), StepSnap::BehindMonocle),
        (step(|f| f.same_monitor = false), StepSnap::OtherMonitor),
    ];

    // Act / Assert
    for (facts, why) in cases {
        assert_eq!(step_snap(&facts), Some(why), "{facts:?}");
        assert!(!step_animates(&facts), "{facts:?}");
    }
}

#[test]
fn the_reasons_about_the_whole_step_come_first() {
    // Arrange -- a window that appears in a rebuild, or in a step that
    // moves too many windows.
    let rebuild = step(|f| (f.rebuild, f.appears) = (true, true));
    let crowded = step(|f| (f.windows_moved, f.appears) = (20, true));

    // Act / Assert
    assert_eq!(step_snap(&rebuild), Some(StepSnap::Rebuild));
    assert_eq!(step_snap(&crowded), Some(StepSnap::TooManyWindows(20)));
}

#[test]
fn each_rule_of_the_apply_time_check_gives_its_own_reason() {
    // Arrange -- a nudge of 4 px is the most that still snaps.
    let nudge = Rect::new(4, 0, 800, 900);
    let cases = [
        (applied(|f| f.system_allows = false), ApplySnap::SystemOff),
        (applied(|f| f.minimized = true), ApplySnap::Minimized),
        (applied(|f| f.maximized = true), ApplySnap::Maximized),
        (applied(|f| f.frames_supported = false), ApplySnap::NoFrames),
        (applied(|f| f.to = nudge), ApplySnap::ShortTravel(4)),
    ];

    // Act / Assert
    for (facts, why) in cases {
        assert_eq!(apply_snap(&facts), Some(why), "{facts:?}");
        assert!(!still_animates(&facts), "{facts:?}");
    }
}

#[test]
fn a_system_that_turns_animations_off_counts_only_while_it_is_followed() {
    // Arrange -- the near miss.
    let ignored = applied(|f| (f.follow_system, f.system_allows) = (false, false));

    // Act / Assert
    assert_eq!(apply_snap(&ignored), None);
}

#[test]
fn every_reason_reads_as_a_cause() {
    // Arrange
    let step_reasons = [
        (StepSnap::Off, "transitions are off"),
        (StepSnap::Rebuild, "the step lays windows out from scratch"),
        (
            StepSnap::TooManyWindows(17),
            "the step moves 17 windows, more than 16",
        ),
        (StepSnap::Appears, "it appears in this step"),
        (
            StepSnap::BehindMonocle,
            "it waits behind the monocle window",
        ),
        (StepSnap::OtherMonitor, "it moves to another monitor"),
    ];
    let apply_reasons = [
        (ApplySnap::DisplaysChanging, "the displays are changing"),
        (ApplySnap::SystemOff, "the system turns animations off"),
        (ApplySnap::Minimized, "it is minimized"),
        (ApplySnap::Maximized, "it is maximized"),
        (ApplySnap::NoFrames, "transitions are off for this session"),
        (ApplySnap::ShortTravel(3), "it travels 3 px, 4 or less"),
    ];

    // Act / Assert
    for (why, text) in step_reasons {
        assert_eq!(why.to_string(), text);
    }
    for (why, text) in apply_reasons {
        assert_eq!(why.to_string(), text);
    }
}
