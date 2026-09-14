//! Keeps the focused monitor pinned while its workspace is empty.
//!
//! When the user navigates to an empty workspace there is no mosaico
//! window able to hold foreground, so Win32 leaves it on another
//! monitor. Any transient overlay that takes and releases foreground, a
//! launcher, Alt+Tab, a UAC prompt, makes Windows hand foreground to
//! whatever is still visible, which is on another monitor. The
//! `Focused` event that follows would rewrite `focused_monitor`, and
//! since `add_and_focus` reads that field, the next window is tiled on
//! the monitor the user just navigated away from.
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
//!   clicking a window is pointing at it, while Windows restoring
//!   foreground after an overlay closes is not accompanied by the
//!   pointer moving there. This is the discriminator that does the
//!   real work.
//! - Focus resolves on the parked monitor.
//! - A window lands on the parked workspace, so it can hold foreground
//!   on its own again.
//! - The deadline passes, purely as a backstop.
//!
//! Known gap: focusing a window on another monitor by keyboard alone,
//! Alt+Tab with the pointer left behind, looks identical to an OS
//! restore and stays suppressed until the deadline.

use std::time::{Duration, Instant};

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

/// What the `Focused` handler should do with an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaimVerdict {
    /// Handle the event normally. The claim is finished.
    Apply,
    /// Ignore the event, and keep the claim for the next one.
    Ignore,
}

/// Returns the monitor whose work area contains the point.
///
/// Used to locate the pointer, so a focus event can be attributed to a
/// person clicking rather than to Windows restoring foreground.
pub(super) fn monitor_at_point(monitors: &[MonitorState], x: i32, y: i32) -> Option<usize> {
    monitors.iter().position(|mon| {
        let area = &mon.work_area;
        x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
    })
}

/// Decides whether a `Focused` event may move `focused_monitor`.
///
/// Kept free of `TilingManager` so it can be table tested: the tiling
/// tests build `MonitorState` values directly and never construct a
/// manager.
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
    // able to hold foreground again.
    if !parked.active_ws().is_empty() {
        return ClaimVerdict::Apply;
    }

    // Focus resolved on the monitor the user navigated to.
    if event_monitor == claim.monitor {
        return ClaimVerdict::Apply;
    }

    // The pointer is on the monitor that just took focus, so treat it
    // as the user going there rather than Windows restoring foreground.
    if cursor_monitor == Some(event_monitor) {
        return ClaimVerdict::Apply;
    }

    ClaimVerdict::Ignore
}

impl super::TilingManager {
    /// Parks `focused_monitor` when the workspace the user just landed
    /// on is empty, so foreground restores elsewhere cannot move it.
    ///
    /// Call this at the end of a navigation, once the new active
    /// workspace is in place.
    pub(super) fn park_focus_if_empty(&mut self) {
        let idx = self.focused_monitor;
        let empty = self
            .monitors
            .get(idx)
            .is_some_and(|mon| mon.active_ws().is_empty());

        if !empty {
            // Landing somewhere with windows resolves any earlier claim.
            self.empty_focus_claim = None;
            return;
        }

        mosaico_core::log_debug!("focus-parked mon {} (workspace is empty)", idx);
        self.empty_focus_claim = Some(EmptyFocusClaim::new(idx, Instant::now()));
    }

    /// Drops any claim, used when monitor indices stop meaning what
    /// they did.
    pub(super) fn clear_focus_claim(&mut self) {
        self.empty_focus_claim = None;
    }

    /// Returns whether a `Focused` event may move `focused_monitor`,
    /// consuming the claim when the event resolves it.
    pub(super) fn focus_event_allowed(&mut self, event_monitor: usize) -> bool {
        let Some(claim) = self.empty_focus_claim else {
            return true;
        };

        let cursor = self.cursor_monitor();
        match judge_focus_event(
            &self.monitors,
            &claim,
            event_monitor,
            cursor,
            Instant::now(),
        ) {
            ClaimVerdict::Apply => {
                self.empty_focus_claim = None;
                true
            }
            ClaimVerdict::Ignore => false,
        }
    }

    /// The monitor the pointer is on, if it is on one.
    fn cursor_monitor(&self) -> Option<usize> {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

        let mut point = POINT::default();

        // SAFETY: GetCursorPos writes the cursor position into a stack
        // allocated POINT and does not modify any window state.
        if unsafe { GetCursorPos(&mut point) }.is_err() {
            return None;
        }

        monitor_at_point(&self.monitors, point.x, point.y)
    }
}
