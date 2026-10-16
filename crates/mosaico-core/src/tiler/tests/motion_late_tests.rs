//! How long the reports of a workspace switch's hide and show are waited
//! for when they come late, behind what was posted to the window: for at
//! most `ASYNC_GRACE` after the show, in the tiler's view of the desktop as
//! in the echo filter.

use crate::config::HidingBehaviour;
use crate::frames::ASYNC_GRACE;
use crate::id::wid;
use crate::tiler::tests::fake_desktop::Queued;
use crate::tiler::tests::manager_support::tiled;
use crate::{Action, WindowEvent};

use super::{REFRESH, animated, swap, two};

#[test]
fn a_minimize_reported_past_the_grace_after_the_show_is_the_users() {
    // Arrange -- workspaces hide windows by minimizing them, and window 1's
    // application is busy, with a frame on its way to it, which does not
    // move it. Away and straight back: the minimize waits behind the frame,
    // and the show behind the minimize.
    let mut config = animated();
    config.layout.hiding = HidingBehaviour::Minimize;
    let (platform, mut manager) = two(&config);
    platform.edit(1, |w| (w.frame_delay, w.absorbs_frames) = (40, true));
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();

    // Act -- well past the grace, window 2 reports where it is; then the
    // application gets to the minimize, which it reports.
    platform.advance(ASYNC_GRACE * 2);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(2) });
    platform.catch_up_through(1, Queued::Minimize);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });

    // Assert -- the tiler no longer takes it for its own, so window 1
    // leaves the layout.
    assert_eq!(tiled(&manager, 0), [2]);
}
