//! Which placements animate, and which snap into place.
//!
//! It is decided twice. The tiler decides for each placement as it
//! finishes a step, from what the step did: [`step_animates`]. The applier
//! checks the placements the tiler let animate again as it applies them,
//! from what the desktop says then: [`still_animates`]. A placement
//! animates only when both say so. One that does not is made at once, as
//! every placement was before transitions.

use crate::Rect;

/// The most windows a step may move for its placements to animate.
///
/// Every frame moves each moving window, and each window moved costs its
/// application a repaint. A step that moves more windows than this snaps
/// them all: the frames would cost more than the transition is worth. It
/// bounds the transition of one step, not how many windows move at once:
/// windows an earlier step set moving may still be on their way.
pub const MAX_ANIMATED_WINDOWS: usize = 16;

/// How far some edge of a window must travel, in pixels, for its
/// placement to animate.
///
/// Moving a window by this much or less is a nudge, as when a gap changes
/// by a pixel or two. Animating it would repaint the window for every
/// frame of the transition without anyone seeing it move.
pub const MIN_TRAVEL: i32 = 4;

/// What the tiler knows about one of its placements as it finishes a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepFacts {
    /// Whether transitions are on: enabled in the config, with a duration
    /// above zero.
    pub enabled: bool,
    /// The step showed the window, took it into the layout or placed it
    /// ahead of the layout before it was shown.
    pub appears: bool,
    /// The window waits behind the window in monocle.
    pub behind_monocle: bool,
    /// The step is a display change, the tiler starting, or the bar
    /// reservation that follows the start.
    pub rebuild: bool,
    /// The window stays on the monitor it is on.
    pub same_monitor: bool,
    /// How many of the step's placements change their window's rect as
    /// the tiler sees it, the desktop's rect. For a moving window that is
    /// where its motion has it, so a target it already has may count, which
    /// can only make a large step snap.
    pub windows_moved: usize,
}

/// Whether the tiler lets a placement animate. It does when all of these
/// hold:
///
/// - Transitions are on: enabled in the config, with a duration above
///   zero. One that would take no time is a placement made at once.
/// - The window does not appear in this step. A window shown, taken in or
///   placed before it was shown appears where it goes, so it is never seen
///   moving there.
/// - It does not wait behind the window in monocle. It goes behind it, out
///   of sight, so it snaps there.
/// - The step is not a display change, the tiler starting, or the bar
///   reservation that follows the start. Monitors and work areas are in
///   flux then, and everything snaps.
/// - The window stays on its monitor. One that moves to another snaps:
///   the other may have another scale, which the application changes to
///   on its own, and core does not know the scales of monitors yet.
/// - The step moves at most [`MAX_ANIMATED_WINDOWS`] windows.
pub fn step_animates(facts: &StepFacts) -> bool {
    facts.enabled
        && !facts.appears
        && !facts.behind_monocle
        && !facts.rebuild
        && facts.same_monitor
        && facts.windows_moved <= MAX_ANIMATED_WINDOWS
}

/// What the applier finds as it applies a placement the tiler let
/// animate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyFacts {
    /// Whether the config follows the system's setting for animations.
    pub follow_system: bool,
    /// Whether the system's setting allows animations.
    pub system_allows: bool,
    /// The window is minimized.
    pub minimized: bool,
    /// The window is maximized.
    pub maximized: bool,
    /// Whether the platform can move windows through frames. One that
    /// cannot places every window at once.
    pub frames_supported: bool,
    /// Where the window is: where its motion has it if it is moving,
    /// otherwise where the desktop has it.
    pub from: Rect,
    /// Where the placement puts it.
    pub to: Rect,
}

/// Whether a placement the tiler let animate still animates as it is
/// applied. It does when all of these hold:
///
/// - The system's setting allows animations, or the config does not follow
///   it. Whoever turned animations off for the whole system gets none.
/// - The window is neither minimized nor maximized. Frames never change
///   either state, so such a window is placed at once, as it always was.
/// - Some edge of the window travels more than [`MIN_TRAVEL`] pixels.
/// - The platform can move windows through frames.
pub fn still_animates(facts: &ApplyFacts) -> bool {
    (facts.system_allows || !facts.follow_system)
        && !facts.minimized
        && !facts.maximized
        && facts.frames_supported
        && travel(facts.from, facts.to) > i64::from(MIN_TRAVEL)
}

/// Whether no edge of a window is more than [`MIN_TRAVEL`] pixels from where
/// it is in `a` to where it is in `b`: a move that small does not animate,
/// and a window that close to where it was placed is taken to be there.
pub fn hardly_apart(a: Rect, b: Rect) -> bool {
    travel(a, b) <= i64::from(MIN_TRAVEL)
}

/// How far the edge of a window that travels furthest goes from `from` to
/// `to`, in pixels.
fn travel(from: Rect, to: Rect) -> i64 {
    let edges = |rect: Rect| {
        let [x, y, width, height] = [rect.x, rect.y, rect.width, rect.height].map(i64::from);
        [x, y, x + width, y + height]
    };
    let (from, to) = (edges(from), edges(to));
    (0..4).map(|i| (to[i] - from[i]).abs()).max().unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/policy_tests.rs"]
mod tests;
