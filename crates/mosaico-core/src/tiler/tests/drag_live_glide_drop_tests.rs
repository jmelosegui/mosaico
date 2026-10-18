//! The glide of the tiles that make room for a dragged window, against
//! reports on the way and a drop mid-glide, and with transitions off.

use super::glide_tests::{frame, placed_at_once, rects};
use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::drag::live::back::manager_support::{center, grab, reported, two_monitors};
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};

#[test]
fn reports_over_the_same_tile_start_no_new_glide() {
    // Arrange -- the same drag twice, the second with reports on the way.
    let glide = |extra: bool| {
        let (platform, mut manager) = two_monitors();
        let (x, y) = center(&platform, 3);
        grab(&platform, &mut manager, 1);
        reported(&platform, &mut manager, 1, (x, y));
        let mut frames = 0;
        if extra {
            frames += rects(&platform, &mut manager, 4, 2).len();
            reported(&platform, &mut manager, 1, (x + 7, y - 3));
            reported(&platform, &mut manager, 1, (x + 14, y - 6));
        }
        while manager.wants_frames() {
            frame(&platform, &mut manager);
            frames += 1;
        }
        (frames, platform.window(4).rect)
    };

    // Act
    let (once, extra) = (glide(false), glide(true));

    // Assert -- the glide ends after as many frames, where it was going.
    assert_eq!(once, extra);
}

#[test]
fn a_drop_mid_glide_leaves_the_tiles_on_their_way() {
    // Arrange -- the same drag twice, the second dropped mid-glide.
    let glide = |drop: bool| {
        let (platform, mut manager) = two_monitors();
        let over = center(&platform, 3);
        grab(&platform, &mut manager, 1);
        reported(&platform, &mut manager, 1, over);
        rects(&platform, &mut manager, 3, 2);
        if drop {
            let pointer = Some(over);
            manager.handle_event(&WindowEvent::Moved {
                window: wid(1),
                pointer,
            });
        }
        let mut both = || {
            frame(&platform, &mut manager);
            (platform.window(3).rect, platform.window(4).rect)
        };
        (0..12).map(|_| both()).collect::<Vec<_>>()
    };

    // Act
    let (held, dropped) = (glide(false), glide(true));

    // Assert -- windows 3 and 4 go the same way, frame by frame.
    assert_eq!(held, dropped);
}

#[test]
fn with_transitions_off_the_tiles_make_room_at_once() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let mut manager = start(&platform, None, &still());
    let over = center(&platform, 3);
    grab(&platform, &mut manager, 1);
    platform.take_calls();

    // Act
    let moving = reported(&platform, &mut manager, 1, over);

    // Assert
    let calls = platform.take_calls();
    assert!(!moving);
    assert!(
        placed_at_once(&calls, 3) && placed_at_once(&calls, 4),
        "{calls:?}"
    );
}
