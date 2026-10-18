//! Dragging a tiled window over another monitor in monocle on the tiling
//! manager, with transitions on: its windows glide to make room as if it
//! were tiled, the drop leaves them there, and they glide back to monocle
//! as the drag leaves, from wherever they are.

use std::time::Duration;

use crate::Action;
use crate::action::Origin;
use crate::tiler::drag::live::back::manager_support::{
    Manager, center, drop_at, grab, moves, report_at, reported, two_monitors,
};
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::settle;

/// Windows 1 and 2 on the first monitor and 3 and 4 on the second, window
/// 3 in monocle there, from its bar, which it moved to fill over time.
fn monocle_on_second() -> (FakePlatform, Manager) {
    let (platform, mut manager) = two_monitors();
    manager.handle_action_from(&Action::ToggleMonocle, Origin::Bar(1));
    manager.flush_pending();
    settle(&platform, &mut manager);
    (platform, manager)
}

#[test]
fn a_monocle_workspace_there_glides_to_make_room_and_the_drop_leaves_its_windows_there() {
    // Arrange -- where window 4 is tiled, behind window 3.
    let (platform, mut manager) = monocle_on_second();
    let over = center(&platform, 4);
    grab(&platform, &mut manager, 1);

    // Act -- over there, and dropped there once they are in place.
    let (made_room, moving) = report_at(&platform, &mut manager, 1, over);
    let dropped = drop_at(&platform, &mut manager, 1, over);

    // Assert
    assert!(
        moves(&made_room, 3) && moves(&made_room, 4),
        "{made_room:?}"
    );
    assert!(moving);
    assert!(!moves(&dropped, 3) && !moves(&dropped, 4), "{dropped:?}");
    assert!(moves(&dropped, 1), "{dropped:?}");
}

#[test]
fn a_monocle_workspace_left_mid_glide_glides_back_from_where_it_is() {
    // Arrange -- over where window 4 is tiled, three frames into the glide
    // that shrinks window 3 to the left half.
    let (platform, mut manager) = monocle_on_second();
    let (home, over) = (center(&platform, 1), center(&platform, 4));
    let (front, behind) = (platform.window(3).rect, platform.window(4).rect);
    grab(&platform, &mut manager, 1);
    reported(&platform, &mut manager, 1, over);
    let step = |platform: &FakePlatform, manager: &mut Manager| {
        platform.advance(Duration::from_millis(16));
        manager.frame();
        platform.window(3).rect
    };
    let mid = (0..3).map(|_| step(&platform, &mut manager)).last();
    let mid = mid.expect("frames");

    // Act -- back over its own slot.
    reported(&platform, &mut manager, 1, home);
    let next = step(&platform, &mut manager);
    settle(&platform, &mut manager);

    // Assert -- window 3 grows back from where it was, and monocle shows
    // it in front again, window 4 back where it was.
    assert!(mid.width < front.width, "{mid:?}");
    assert!(
        mid.width < next.width && next.width < front.width,
        "{next:?}"
    );
    assert_eq!(
        (platform.window(3).rect, platform.window(4).rect),
        (front, behind)
    );
}
