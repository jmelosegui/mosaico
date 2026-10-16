//! Whether a maximized window placed in a step is still maximized after it.

use std::time::Instant;

use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::tiler::tests::fake_desktop::{FakeDesktop, FakeWindow};
use crate::tiler::{Effect, Timing};
use crate::{Rect, WindowId};

use super::Plan;

/// A desktop with maximized window 1, whose changes are posted or not.
fn maximized(posted: bool) -> FakeDesktop {
    let mut desktop = FakeDesktop::default();
    let mut window = FakeWindow::app(0);
    window.maximized = true;
    desktop.windows.insert(wid(1), window);
    if posted {
        desktop.posting.insert(wid(1));
    }
    desktop
}

/// Places `window` in `plan`.
fn place(plan: &mut Plan, window: WindowId) {
    let slot = Rect::new(8, 8, 948, 1064);
    plan.emit(Effect::snap(window, slot, Timing::Now));
}

#[test]
fn a_maximized_window_placed_in_view_is_no_longer_maximized() {
    for posted in [false, true] {
        // Arrange
        let desktop = maximized(posted);
        let mut plan = Plan::new(&desktop, Instant::now());

        // Act
        place(&mut plan, wid(1));

        // Assert
        assert!(!plan.is_maximized(wid(1)), "posted: {posted}");
    }
}

#[test]
fn a_maximized_window_placed_out_of_view_stays_maximized_while_its_changes_are_posted() {
    let window = wid(1);
    let hide = Effect::Hide {
        window,
        how: HidingBehaviour::Hide,
    };
    for out_of_view in [hide, Effect::Minimize { window }] {
        for posted in [false, true] {
            // Arrange
            let desktop = maximized(posted);
            let mut plan = Plan::new(&desktop, Instant::now());

            // Act -- placed, then hidden or minimized in the same step.
            place(&mut plan, window);
            plan.emit(out_of_view.clone());

            // Assert -- the platform may leave it maximized when the
            // placement is posted, and it ends the maximized state when it
            // is made at once.
            assert_eq!(plan.is_maximized(window), posted, "{out_of_view:?}");
        }
    }
}
