use super::super::MonitorState;
use super::super::empty_focus::{
    ClaimVerdict, EMPTY_FOCUS_CLAIM_TTL, EmptyFocusClaim, judge_focus_event, monitor_at_point,
};
use super::make_monitor;
use mosaico_core::Rect;
use std::time::{Duration, Instant};

/// Monitor 0 on the left holding the window that has foreground, and
/// monitor 1 on the right parked on an empty workspace. This is the
/// arrangement from the reproduction: the user pressed alt+l then alt+8.
fn parked_on_empty() -> (Vec<MonitorState>, EmptyFocusClaim, Instant) {
    let mut left = make_monitor(0);
    left.work_area = Rect::new(0, 0, 3840, 2024);
    left.workspaces[0].add(0x40F78);

    let mut right = make_monitor(1);
    right.work_area = Rect::new(3840, 0, 3840, 2024);
    right.active_workspace = 7;

    let now = Instant::now();
    let claim = EmptyFocusClaim::new(1, now);

    (vec![left, right], claim, now)
}

/// Pointer left on the parked monitor, which is where it is after the
/// user navigates there.
const CURSOR_ON_PARKED: Option<usize> = Some(1);

#[test]
fn a_foreground_restore_to_another_monitor_is_ignored() {
    // Arrange
    let (monitors, claim, now) = parked_on_empty();

    // Act -- a transient overlay took foreground and closed, so Windows
    // handed it to a window that is still visible, on monitor 0. The
    // pointer never went there.
    let verdict = judge_focus_event(&monitors, &claim, 0, CURSOR_ON_PARKED, now);

    // Assert
    assert_eq!(verdict, ClaimVerdict::Ignore);
}

#[test]
fn a_click_on_the_other_monitor_is_honoured() {
    // Arrange
    let (monitors, claim, now) = parked_on_empty();

    // Act -- same event, except the pointer is on the monitor that took
    // focus, which is what clicking a window there looks like.
    let verdict = judge_focus_event(&monitors, &claim, 0, Some(0), now);

    // Assert -- this is the failure mode of PR #25, where the latch made
    // every click on the other monitor unreachable.
    assert_eq!(verdict, ClaimVerdict::Apply);
}

#[test]
fn focus_resolving_on_the_parked_monitor_ends_the_claim() {
    // Arrange
    let (monitors, claim, now) = parked_on_empty();

    // Act
    let verdict = judge_focus_event(&monitors, &claim, 1, CURSOR_ON_PARKED, now);

    // Assert -- the user is focused where they navigated to, so there
    // is nothing left to protect.
    assert_eq!(verdict, ClaimVerdict::Apply);
}

#[test]
fn a_window_landing_on_the_parked_workspace_ends_the_claim() {
    // Arrange
    let (mut monitors, claim, now) = parked_on_empty();
    monitors[1].workspaces[7].add(0xABC);

    // Act
    let verdict = judge_focus_event(&monitors, &claim, 0, CURSOR_ON_PARKED, now);

    // Assert -- the workspace can hold foreground on its own now.
    assert_eq!(verdict, ClaimVerdict::Apply);
}

#[test]
fn an_expired_claim_is_ignored() {
    // Arrange
    let (monitors, claim, now) = parked_on_empty();

    // Act
    let later = now + EMPTY_FOCUS_CLAIM_TTL + Duration::from_millis(1);
    let verdict = judge_focus_event(&monitors, &claim, 0, CURSOR_ON_PARKED, later);

    // Assert -- a claim that nothing resolved must not outlive its
    // deadline, or it becomes the mirror image of the bug it fixes.
    assert_eq!(verdict, ClaimVerdict::Apply);
}

#[test]
fn a_claim_survives_well_past_the_old_three_second_draft() {
    // Arrange -- the first version of this guard expired after three
    // seconds, which elapsed between arriving on the empty workspace
    // and the overlay stealing focus, so it never fired.
    let (monitors, claim, now) = parked_on_empty();

    // Act
    let later = now + Duration::from_secs(20);
    let verdict = judge_focus_event(&monitors, &claim, 0, CURSOR_ON_PARKED, later);

    // Assert
    assert_eq!(verdict, ClaimVerdict::Ignore);
}

#[test]
fn a_claim_for_a_monitor_that_no_longer_exists_is_ignored() {
    // Arrange -- a display change rebuilt the monitor list and left
    // fewer monitors than the claim was made against.
    let (_, claim, now) = parked_on_empty();
    let monitors = vec![make_monitor(0)];

    // Act
    let verdict = judge_focus_event(&monitors, &claim, 0, CURSOR_ON_PARKED, now);

    // Assert
    assert_eq!(verdict, ClaimVerdict::Apply);
}

#[test]
fn an_unknown_cursor_position_does_not_count_as_a_click() {
    // Arrange -- GetCursorPos failed, or the pointer is over the bar
    // rather than any work area.
    let (monitors, claim, now) = parked_on_empty();

    // Act
    let verdict = judge_focus_event(&monitors, &claim, 0, None, now);

    // Assert -- without evidence that the user pointed at the monitor,
    // the event is treated as a restore.
    assert_eq!(verdict, ClaimVerdict::Ignore);
}

// -- monitor_at_point --

#[test]
fn the_pointer_is_placed_on_the_monitor_whose_work_area_holds_it() {
    // Arrange
    let (monitors, _, _) = parked_on_empty();

    // Act / Assert
    assert_eq!(monitor_at_point(&monitors, 100, 100), Some(0));
    assert_eq!(monitor_at_point(&monitors, 5000, 100), Some(1));
}

#[test]
fn a_pointer_outside_every_work_area_is_placed_nowhere() {
    // Arrange
    let (monitors, _, _) = parked_on_empty();

    // Act / Assert -- below both work areas, for example over a taskbar
    // that the work area excludes.
    assert_eq!(monitor_at_point(&monitors, 100, 9000), None);
}
