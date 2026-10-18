//! What a drag over another monitor asks of the platform with transitions
//! on: the tiles there glide to make room each time the target changes.

use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{animated, settle, start};
use crate::tiler::tests::metrics_support::{Measured, on_settled};

use super::drag::{drag, path_over};

/// Two windows on each of two monitors, with transitions on; window 1 is
/// dragged over window 3 on the second, on over window 4, and on to the
/// gap above the tiles there, the windows settling after each report, and
/// dropped there: the tiles there glide to make room each time, and the
/// drop leaves them where they are.
pub(super) fn over_another_monitor() -> Measured {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    let path = path_over(&platform);
    let dragged = platform.clone();
    on_settled((platform, manager), |manager| {
        drag(&dragged, manager, 1, &path, true);
    })
}
