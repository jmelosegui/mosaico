//! Echoes of hiding on platforms whose ways of hiding report something
//! other than the usual events.

use std::time::Instant;

use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::{HidingEvents, HidingSupport, WindowEvent};

use super::*;

/// Hiding a window reports it hidden; showing it again reports nothing.
const REPORTS_HIDDEN: HidingEvents = HidingEvents {
    reports_hidden: true,
    reports_restored: false,
};

fn hide(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Hide {
        window: wid(window),
        how,
    }
}

fn show(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Show {
        window: wid(window),
        how,
    }
}

#[test]
fn a_hide_report_comes_from_the_application_where_hiding_reports_nothing() {
    // Arrange -- on this platform no way of hiding reports anything.
    let now = Instant::now();
    let mut ex = Expectations::new(HidingSupport::new(|_| Some(HidingEvents::default())));
    ex.applied(&hide(1, HidingBehaviour::Hide), now);

    // Act
    let echo = ex.echo(&WindowEvent::Hidden { window: wid(1) }, now);

    // Assert
    assert_eq!(echo, None);
}

#[test]
fn a_hide_report_is_an_echo_where_even_cloaking_reports_one() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::new(HidingSupport::new(|_| Some(REPORTS_HIDDEN)));
    ex.applied(&hide(1, HidingBehaviour::Cloak), now);

    // Act / Assert
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(1) }, now)
            .is_some()
    );
    assert!(
        ex.echo(&WindowEvent::Minimized { window: wid(1) }, now)
            .is_some()
    );
}

#[test]
fn a_restore_is_expected_only_where_showing_reports_one() {
    // Arrange -- here showing a hidden window reports a restore, and
    // showing a minimized one does not.
    let now = Instant::now();
    let mut ex = Expectations::new(HidingSupport::new(|how| {
        Some(HidingEvents {
            reports_hidden: true,
            reports_restored: how == HidingBehaviour::Hide,
        })
    }));
    ex.applied(&hide(1, HidingBehaviour::Minimize), now);
    ex.applied(&hide(2, HidingBehaviour::Hide), now);

    // Act
    ex.applied(&show(1, HidingBehaviour::Minimize), now);
    ex.applied(&show(2, HidingBehaviour::Hide), now);

    // Assert
    assert_eq!(
        ex.echo(&WindowEvent::Restored { window: wid(1) }, now),
        None
    );
    assert!(
        ex.echo(&WindowEvent::Restored { window: wid(2) }, now)
            .is_some()
    );
}

#[test]
fn hiding_a_way_the_platform_lacks_expects_no_report() {
    // Arrange -- the platform cannot minimize windows to hide them, so
    // the hide changed nothing.
    let now = Instant::now();
    let mut ex = Expectations::new(HidingSupport::new(|how| {
        (how != HidingBehaviour::Minimize).then_some(REPORTS_HIDDEN)
    }));
    ex.applied(&hide(1, HidingBehaviour::Minimize), now);

    // Act
    let echo = ex.echo(&WindowEvent::Minimized { window: wid(1) }, now);

    // Assert
    assert_eq!(echo, None);
}
