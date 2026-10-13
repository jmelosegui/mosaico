use std::time::{Duration, Instant};

use crate::WindowEvent;
use crate::config::HidingBehaviour;

use crate::id::wid;

use super::*;

fn hide(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Hide {
        window: wid(window),
        how,
    }
}

fn focus(window: u64) -> Effect {
    Effect::Focus {
        window: wid(window),
        move_cursor: false,
    }
}

#[test]
fn hide_report_for_a_window_we_hid_is_an_echo() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Hide), now);

    // Act / Assert
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(1) }, now)
            .is_some()
    );
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(2) }, now)
            .is_none()
    );
}

#[test]
fn minimize_report_for_a_window_we_minimized_is_an_echo() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Minimize), now);

    // Act / Assert
    assert!(
        ex.echo(&WindowEvent::Minimized { window: wid(1) }, now)
            .is_some()
    );
}

#[test]
fn hide_report_for_a_cloaked_window_comes_from_the_application() {
    // Arrange -- cloaking reports nothing by itself.
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Cloak), now);

    // Act / Assert
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(1) }, now)
            .is_none()
    );
}

#[test]
fn showing_a_window_ends_the_hide_expectation() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Hide), now);

    // Act
    ex.applied(
        &Effect::Show {
            window: wid(1),
            how: HidingBehaviour::Hide,
        },
        now,
    );

    // Assert
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(1) }, now)
            .is_none()
    );
}

#[test]
fn restoring_a_window_we_minimized_is_an_echo_once() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Minimize), now);
    ex.applied(
        &Effect::Show {
            window: wid(1),
            how: HidingBehaviour::Minimize,
        },
        now,
    );

    // Act / Assert -- the user restoring it later is real.
    assert!(
        ex.echo(&WindowEvent::Restored { window: wid(1) }, now)
            .is_some()
    );
    assert!(
        ex.echo(&WindowEvent::Restored { window: wid(1) }, now)
            .is_none()
    );
}

#[test]
fn showing_a_window_we_did_not_minimize_expects_no_restore() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();

    // Act
    ex.applied(
        &Effect::Show {
            window: wid(1),
            how: HidingBehaviour::Minimize,
        },
        now,
    );

    // Assert
    assert!(
        ex.echo(&WindowEvent::Restored { window: wid(1) }, now)
            .is_none()
    );
}

#[test]
fn focus_report_for_an_earlier_target_is_stale() {
    // Arrange -- the user pressed alt+l twice in quick succession.
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&focus(1), now);
    ex.applied(&focus(2), now + Duration::from_millis(50));
    let later = now + Duration::from_millis(300);

    // Act / Assert
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(1) }, later)
            .is_some()
    );
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(2) }, later)
            .is_none()
    );
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(3) }, later)
            .is_none()
    );
}

#[test]
fn stale_focus_reports_expire() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&focus(1), now);
    ex.applied(&focus(2), now);

    // Act -- well after the reports could still be in flight.
    let later = now + FOCUS_ECHO_TTL + Duration::from_millis(1);

    // Assert
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(1) }, later)
            .is_none()
    );
}

#[test]
fn other_focus_reports_wait_while_a_focus_change_is_held() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.held(&focus(2));

    // Act / Assert
    assert!(ex.is_focus_held());
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(1) }, now)
            .is_some()
    );
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(2) }, now)
            .is_none()
    );
}

#[test]
fn applying_the_held_focus_change_releases_it() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.held(&focus(2));

    // Act
    ex.applied(&focus(2), now);

    // Assert
    assert!(!ex.is_focus_held());
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(3) }, now)
            .is_none()
    );
}

#[test]
fn focus_report_for_a_window_just_hidden_is_an_echo() {
    // Arrange -- the switch hid the window that had focus.
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Cloak), now);

    // Act / Assert
    let soon = now + Duration::from_millis(100);
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(1) }, soon)
            .is_some()
    );
    let late = now + HIDE_FOCUS_ECHO_TTL;
    assert!(
        ex.echo(&WindowEvent::Focused { window: wid(1) }, late)
            .is_none(),
        "picking it from the taskbar later is the user"
    );
}

#[test]
fn destroyed_windows_are_forgotten() {
    // Arrange
    let now = Instant::now();
    let mut ex = Expectations::default();
    ex.applied(&hide(1, HidingBehaviour::Hide), now);

    // Act
    let destroyed = ex.echo(&WindowEvent::Destroyed { window: wid(1) }, now);

    // Assert
    assert!(destroyed.is_none());
    assert!(
        ex.echo(&WindowEvent::Hidden { window: wid(1) }, now)
            .is_none()
    );
}
