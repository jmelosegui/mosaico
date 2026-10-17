//! Keeps the focused monitor pinned while its workspace is empty.
//!
//! When the user navigates to an empty workspace there is no managed
//! window able to hold focus, so the OS leaves it on another monitor.
//! Any transient overlay that takes and releases focus, a launcher, the
//! task switcher, a permission prompt, makes the OS hand focus to
//! whatever is still visible, which is on another monitor. The focus
//! event that follows would rewrite `focused_monitor`, and since
//! `add_and_focus` reads that field, the next window is tiled on the
//! monitor the user just navigated away from.
//!
//! A claim records the monitor the user navigated to. While it is live,
//! focus events naming other monitors do not move `focused_monitor`.
//!
//! The claim has to end, otherwise it becomes the mirror image of the
//! bug it fixes: PR #25 pinned the monitor with a latch that nothing
//! reachable could clear, so clicking a window on the other monitor was
//! ignored, along with every click after it.
//!
//! What ends a claim, in order of how often it happens:
//!
//! - The pointer is on the monitor the focus event names. Someone
//!   clicking a window is pointing at it, while the OS restoring focus
//!   after an overlay closes is not accompanied by the pointer moving
//!   there. This is the discriminator that does the real work.
//! - Focus resolves on the parked monitor.
//! - A window lands on the parked workspace, so it can hold focus on
//!   its own again.
//! - The deadline passes, purely as a backstop.
//!
//! Known gap: focusing a window on another monitor by keyboard alone,
//! with the pointer left behind, looks identical to an OS restore and
//! stays suppressed until the deadline.

use std::time::{Duration, Instant};

use crate::Rect;

use super::MonitorState;

/// How long a claim survives when nothing resolves it.
///
/// A backstop, not the primary mechanism. The conditions above end a
/// claim within one user action in every case they cover, so this only
/// matters when none of them is reached. It is deliberately long: an
/// earlier draft used three seconds, which expired in the gap between
/// arriving on the empty workspace and opening the launcher, so the
/// guard never fired and the bug reproduced unchanged.
pub(super) const EMPTY_FOCUS_CLAIM_TTL: Duration = Duration::from_secs(30);

/// A monitor parked on an empty workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EmptyFocusClaim {
    /// The monitor the user navigated to.
    pub(super) monitor: usize,
    /// When the claim stops being honoured.
    pub(super) expires_at: Instant,
}

impl EmptyFocusClaim {
    pub(super) fn new(monitor: usize, now: Instant) -> Self {
        Self {
            monitor,
            expires_at: now + EMPTY_FOCUS_CLAIM_TTL,
        }
    }
}

/// What the focus handler should do with an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaimVerdict {
    /// Handle the event normally. The claim is finished.
    Apply,
    /// Ignore the event, and keep the claim for the next one.
    Ignore,
}

/// Returns the monitor whose work area contains the point, or the strip
/// above it that the bar is reserved in.
///
/// Used to locate the pointer, so a focus event can be attributed to a
/// person clicking rather than to the OS restoring focus. A person who
/// just clicked a bar is pointing at that bar's monitor.
pub(super) fn monitor_at_point(monitors: &[MonitorState], x: i32, y: i32) -> Option<usize> {
    monitors.iter().position(|mon| {
        let (area, os) = (mon.work_area, mon.os_work_area);
        let strip = Rect::new(os.x, os.y, os.width, area.y - os.y);
        holds(&area, x, y) || holds(&strip, x, y)
    })
}

/// Whether `area` contains the point.
fn holds(area: &Rect, x: i32, y: i32) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

/// Decides whether a focus event may move `focused_monitor`.
pub(super) fn judge_focus_event(
    monitors: &[MonitorState],
    claim: &EmptyFocusClaim,
    event_monitor: usize,
    cursor_monitor: Option<usize>,
    now: Instant,
) -> ClaimVerdict {
    // Nothing resolved the claim in time.
    if now >= claim.expires_at {
        return ClaimVerdict::Apply;
    }

    // The monitor list was rebuilt under us and the index no longer
    // means what it did. Callers clear the claim on display changes,
    // this is the backstop.
    let Some(parked) = monitors.get(claim.monitor) else {
        return ClaimVerdict::Apply;
    };

    // A window arrived on the parked workspace, so there is something
    // able to hold focus again.
    if !parked.active_ws().is_empty() {
        return ClaimVerdict::Apply;
    }

    // Focus resolved on the monitor the user navigated to.
    if event_monitor == claim.monitor {
        return ClaimVerdict::Apply;
    }

    // The pointer is on the monitor that just took focus, so treat it
    // as the user going there rather than the OS restoring focus.
    if cursor_monitor == Some(event_monitor) {
        return ClaimVerdict::Apply;
    }

    ClaimVerdict::Ignore
}
