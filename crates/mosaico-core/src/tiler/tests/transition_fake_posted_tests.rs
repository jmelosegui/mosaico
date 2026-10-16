//! When the fake platform queues a window's changes behind its frames,
//! as `FrameControl::posts_behind` says: never for a window never given a
//! frame, and past the grace for as long as one has not been seen to move
//! since the last thing posted to it, or its application hangs.

use super::*;

#[test]
fn a_window_not_responding_with_nothing_posted_is_changed_at_once() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.edit(1, |w| w.hung = true);
    let target = Rect::new(400, 100, 640, 480);

    // Act
    let posted = platform.posts_behind(wid(1));
    platform.place(wid(1), target).expect("placing works");

    // Assert -- as before transitions.
    assert!(!posted);
    assert_eq!(platform.window(1).rect, target);
}

#[test]
fn posts_behind_holds_from_a_frame_until_the_window_is_seen_to_move_past_the_grace() {
    // Arrange
    let platform = frame_queued();
    let queued_frame = platform.posts_behind(wid(1));

    // Act
    platform.advance(ASYNC_GRACE * 2);
    let unmoved = platform.posts_behind(wid(1));
    platform.catch_up(1);
    let moved = platform.posts_behind(wid(1));

    // Assert
    assert!(queued_frame && unmoved);
    assert!(!moved);
}

#[test]
fn a_window_that_took_its_frame_without_moving_gets_its_changes_queued_for_good() {
    // Arrange -- it rounds its size back, so it never moves.
    let platform = frame_queued();
    platform.edit(1, |w| w.absorbs_frames = true);
    platform.catch_up(1);
    platform.advance(ASYNC_GRACE * 4);

    // Act
    platform.minimize(wid(1)).expect("the window exists");

    // Assert -- which is safe: it lands once the window gets to it.
    assert_eq!(queued(&platform), vec![Queued::Minimize]);
    platform.catch_up(1);
    assert!(platform.window(1).minimized);
}

#[test]
fn a_placement_queued_behind_a_frame_ends_a_maximized_state() {
    // Arrange -- the user maximized the window while the frame was queued.
    let platform = frame_queued();
    platform.edit(1, |w| w.maximized = true);
    let target = Rect::new(400, 100, 640, 480);

    // Act
    platform.place(wid(1), target).expect("placing works");
    let maximized = platform.window(1).maximized;
    platform.catch_up(1);

    // Assert
    assert!(maximized, "the placement was read back as done");
    assert!(!platform.window(1).maximized);
    assert_eq!(platform.window(1).rect, target);
}

#[test]
fn putting_a_window_behind_another_is_queued_behind_its_frame() {
    // Arrange -- window 1 has a frame queued, and is put in front of
    // window 2, which had nothing posted, so at once.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    platform.edit(1, |w| w.frame_delay = 3);
    post(&platform, &[frame(120, true)]);
    platform
        .put_behind(wid(2), wid(1))
        .expect("both windows exist");
    let in_front = |a: u64, b: u64| platform.shared.desktop.borrow().in_front(wid(a), wid(b));

    // Act -- window 1 goes back behind window 2.
    platform
        .put_behind(wid(1), wid(2))
        .expect("both windows exist");
    let at_once = in_front(2, 1);
    platform.catch_up(1);

    // Assert -- once its application gets to it.
    assert!(!at_once);
    assert!(in_front(2, 1));
}

#[test]
fn a_placement_past_the_grace_queues_behind_one_a_hung_window_has_not_got_to() {
    // Arrange -- the frame landed, then the application hung with the
    // motion's placement still queued.
    let platform = frame_queued();
    platform.catch_up(1);
    platform.edit(1, |w| w.hung = true);
    let old = Rect::new(400, 100, 640, 480);
    platform.place(wid(1), old).expect("placing works");
    platform.advance(ASYNC_GRACE * 2);

    // Act
    let new = Rect::new(800, 100, 640, 480);
    platform.place(wid(1), new).expect("placing works");
    platform.catch_up(1);

    // Assert -- the later placement lands last.
    assert_eq!(platform.window(1).rect, new);
}

#[test]
fn a_show_past_the_grace_queues_behind_a_hide_a_hung_window_has_not_got_to() {
    // Arrange -- the frame landed, then the application hung with a
    // hide queued.
    let platform = frame_queued();
    platform.catch_up(1);
    platform.edit(1, |w| w.hung = true);
    let how = HidingBehaviour::Hide;
    platform.hide(wid(1), how).expect("hiding works");
    platform.advance(ASYNC_GRACE * 2);

    // Act
    platform.show(wid(1), how).expect("showing works");
    platform.catch_up(1);

    // Assert -- shown, not left hidden on the workspace in view.
    assert!(platform.window(1).visible);
}
