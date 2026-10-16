//! The fake platform's maximized windows placed behind what is queued for
//! them, and the windows it forgets once transitions are off.

use super::*;

#[test]
fn a_hung_window_out_of_view_stays_maximized_and_a_show_restores_its_old_bounds() {
    // Arrange -- maximized and hidden, its frame queued, its application
    // hung.
    let platform = frame_queued();
    let normal = platform.window(1).normal;
    platform.edit(1, |w| {
        (w.maximized, w.visible, w.hung) = (true, false, true)
    });
    let target = Rect::new(400, 100, 640, 480);

    // Act
    platform.place(wid(1), target).expect("placing works");
    platform.catch_up(1);
    let placed = platform.window(1);
    platform
        .show(wid(1), HidingBehaviour::Hide)
        .expect("showing works");
    platform.catch_up(1);
    let shown = platform.window(1);

    // Assert -- not its slot, but where it was before it was maximized.
    assert!(placed.maximized && placed.rect == target, "{placed:?}");
    assert!(shown.visible && !shown.maximized, "{shown:?}");
    assert_eq!(shown.rect, normal);
}

#[test]
fn a_maximized_window_placed_while_its_hide_is_queued_stays_maximized() {
    // Arrange -- maximized and shown, its frame queued, then hidden behind
    // it.
    let platform = frame_queued();
    platform.edit(1, |w| w.maximized = true);
    platform
        .hide(wid(1), HidingBehaviour::Hide)
        .expect("hiding works");
    let target = Rect::new(400, 100, 640, 480);

    // Act
    platform.place(wid(1), target).expect("placing works");
    platform.catch_up(1);

    // Assert -- hidden, and moved without ending its maximized state.
    let placed = platform.window(1);
    assert!(placed.maximized && !placed.visible, "{placed:?}");
    assert_eq!(placed.rect, target);
}

#[test]
fn a_window_settled_once_transitions_are_off_is_changed_at_once_again() {
    // Arrange -- it rounds its size back, so it never moves, and its
    // changes stay queued.
    let platform = frame_queued();
    platform.edit(1, |w| w.absorbs_frames = true);
    platform.catch_up(1);
    platform.advance(ASYNC_GRACE * 2);

    // Act
    platform.settle_responding();
    let posted = platform.posts_behind(wid(1));
    platform.minimize(wid(1)).expect("the window exists");

    // Assert
    assert!(!posted);
    assert!(platform.window(1).minimized);
    assert!(queued(&platform).is_empty());
    assert!(platform.calls().contains(&Call::SettleResponding));
}
