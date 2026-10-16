//! What `ordered` notes of the changes it makes, and how a repaint is
//! made, on hidden windows of the test's own.

use mosaico_core::Error;
use mosaico_core::frames::{ASYNC_GRACE, Frame, FrameControl};
use windows::Win32::Graphics::Gdi::RDW_UPDATENOW;

use crate::window::test_windows::HiddenWindow;

use super::*;

/// The first frame of a motion of `window`.
fn first_frame(window: WindowId) -> Frame {
    Frame {
        window,
        rect: Rect::new(40, 30, 120, 90),
        first: true,
    }
}

/// Posts each window its first frame, which a window of the test's
/// thread takes at once.
fn post_first_frames(control: &Win32WindowControl, windows: &[&HiddenWindow]) {
    let frames: Vec<Frame> = windows.iter().map(|w| first_frame(w.id())).collect();
    control
        .place_frames(&frames, &mut Vec::new())
        .expect("frames are supported");
}

#[test]
fn a_change_it_posted_is_noted_and_one_that_failed_is_not() {
    // Arrange -- two windows that took their frames at once, so only the
    // grace has the changes made right after posted.
    let (Some(one), Some(two)) = (HiddenWindow::new(), HiddenWindow::new()) else {
        return;
    };
    let control = Win32WindowControl::default();
    post_first_frames(&control, &[&one, &two]);

    // Act -- a change to each, the second failing.
    let mut asked = Vec::new();
    let made = control.ordered(one.id(), |posted| {
        asked.push(posted);
        Ok(())
    });
    let failed = control.ordered(two.id(), |posted| {
        asked.push(posted);
        Err(Error::Unsupported("a test change"))
    });

    // Assert -- both were posted. Long after the grace, the one noted
    // still holds, as the window has not moved since, and the frames,
    // which moved them, do not.
    let later = Instant::now() + ASYNC_GRACE * 4;
    assert_eq!(asked, [true, true]);
    assert!(made.is_ok() && failed.is_err());
    assert!(control.posts_behind_at(one.id(), later));
    assert!(!control.posts_behind_at(two.id(), later));
}

#[test]
fn a_repaint_waits_for_the_paint_only_while_nothing_posted_may_be_queued() {
    // Arrange -- a frame posted to one window, nothing to the other.
    let (Some(one), Some(two)) = (HiddenWindow::new(), HiddenWindow::new()) else {
        return;
    };
    let control = Win32WindowControl::default();
    post_first_frames(&control, &[&one]);

    // Act
    let (posted, made) = (
        control.repaint_flags(one.id()),
        control.repaint_flags(two.id()),
    );

    // Assert
    assert_eq!(posted & RDW_UPDATENOW, Default::default());
    assert_eq!(made & RDW_UPDATENOW, RDW_UPDATENOW);
    assert_eq!(posted | RDW_UPDATENOW, made);
}
