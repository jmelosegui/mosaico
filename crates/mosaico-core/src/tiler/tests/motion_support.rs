//! What the transition tests read from the calls the fake platform
//! recorded.

use crate::Rect;
use crate::frames::Frame;
use crate::tiler::tests::fake_platform::Call;

/// The placements among `calls`, in order.
pub(super) fn placements(calls: &[Call]) -> Vec<(u64, Rect)> {
    let placed = |call: &Call| match *call {
        Call::Place(window, rect) => Some((window.raw(), rect)),
        _ => None,
    };
    calls.iter().filter_map(placed).collect()
}

/// The frames posted among `calls`, call by call.
pub(super) fn posted(calls: &[Call]) -> Vec<Vec<Frame>> {
    let frames = |call: &Call| match call {
        Call::PlaceFrames(frames) => Some(frames.clone()),
        _ => None,
    };
    calls.iter().filter_map(frames).collect()
}

/// The frame step, counting from 1, in which `window` was placed.
pub(super) fn placed_in(steps: &[Vec<Call>], window: u64) -> Option<usize> {
    let placed = |calls: &Vec<Call>| placements(calls).iter().any(|&(w, _)| w == window);
    steps.iter().position(placed).map(|step| step + 1)
}

/// Whether `calls` draw the border of window `id` around `rect`.
pub(super) fn bordered_at(calls: &[Call], id: u64, rect: Rect) -> bool {
    let at =
        |call: &Call| matches!(call, Call::ShowBorder(w, s) if w.raw() == id && s.rect == rect);
    calls.iter().any(at)
}
