use crate::Rect;
use crate::id::wid;
use crate::motion::{ApplyFacts, ApplySnap, StepFacts, StepSnap};

use super::super::SnapLine;

#[path = "journal_snap_step_tests.rs"]
mod step_tests;

const FROM: Rect = Rect {
    x: 0,
    y: 0,
    width: 800,
    height: 900,
};
const TO: Rect = Rect { x: 400, ..FROM };

/// A placement of a step of two windows that may animate.
fn step() -> StepFacts {
    StepFacts {
        enabled: true,
        appears: false,
        behind_monocle: false,
        rebuild: false,
        same_monitor: true,
        windows_moved: 2,
    }
}

/// A placement from `from` to `to` that still animates as it is applied,
/// unless it stays where it is.
fn applied(from: Rect, to: Rect) -> ApplyFacts {
    ApplyFacts {
        follow_system: true,
        system_allows: true,
        minimized: false,
        maximized: false,
        frames_supported: true,
        from,
        to,
    }
}

#[test]
fn a_window_the_step_moves_elsewhere_gets_the_reason_it_snaps() {
    // Arrange
    let facts = StepFacts {
        same_monitor: false,
        ..step()
    };

    // Act
    let line = SnapLine::stepped(wid(1), &facts, true);

    // Assert
    let why = StepSnap::OtherMonitor;
    let window = wid(1);
    assert_eq!(line, Some(SnapLine { window, why }));
}

#[test]
fn the_step_decision_has_no_line_for_a_window_that_animates_stays_or_snaps_with_its_step() {
    // Arrange
    let crowded = StepFacts {
        windows_moved: 17,
        ..step()
    };
    let elsewhere = StepFacts {
        same_monitor: false,
        ..step()
    };

    // Act
    let lines = [
        SnapLine::stepped(wid(1), &step(), true),
        SnapLine::stepped(wid(1), &crowded, true),
        SnapLine::stepped(wid(1), &elsewhere, false),
    ];

    // Assert
    assert_eq!(lines, [None, None, None]);
}

#[test]
fn a_placement_that_snaps_as_it_is_applied_gets_its_reason_unless_its_window_stays() {
    // Arrange -- the system turns animations off.
    let off = |from, to| ApplyFacts {
        system_allows: false,
        ..applied(from, to)
    };

    // Act
    let animates = SnapLine::applied(wid(1), &applied(FROM, TO));
    let snaps = SnapLine::applied(wid(1), &off(FROM, TO));
    let stays = SnapLine::applied(wid(1), &off(TO, TO));

    // Assert
    let why = ApplySnap::SystemOff;
    let window = wid(1);
    assert_eq!(animates, None);
    assert_eq!(snaps, Some(SnapLine { window, why }));
    assert_eq!(stays, None);
}
