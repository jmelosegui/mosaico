//! The ratio a split gets from where its edge was dropped.

use super::min_limits::fit;
use super::split_edges::Border;
use super::{MAX_SPLIT_RATIO, MIN_SPLIT_RATIO};

/// The ratio that puts a split a window borders as `border` where its edge
/// is, `at` from the start of the split's area, which is `len` long that
/// way, with `gap` between windows. `limits` keep the edge where no window
/// on either side is left below its minimum size, see
/// [`limits`](super::min_limits::limits).
pub(super) fn ratio_at(
    border: Border,
    at: i32,
    len: i32,
    gap: i32,
    limits: Option<(i32, i32)>,
) -> Option<f64> {
    if len <= 0 {
        return None;
    }
    let half = gap / 2;
    // What the layout makes of the ratio: where the split is from the start
    // of its area, or how wide the centered master is.
    let size = match border {
        Border::First => at + half,
        Border::Rest => at - half,
        Border::MasterLeft => len - 2 * at,
        Border::MasterRight => 2 * at - len,
        Border::LeftColumn => len - 2 * (at + gap),
        Border::RightColumn => 2 * (at - gap) - len,
        // Down a stack, `stack_edge` works the heights out.
        Border::Above | Border::Below => return None,
    };
    // Half a pixel more, as the layout rounds down, so that the edge lands
    // on the pixel it was dropped at.
    let ratio = (f64::from(fit(size, limits)) + 0.5) / f64::from(len);
    Some(ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO))
}
