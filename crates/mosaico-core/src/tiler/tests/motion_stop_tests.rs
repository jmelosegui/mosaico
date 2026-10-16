//! Motions the tiler stopping ends, with or without a frame on its way,
//! and the windows a display change stopped: every window is placed in its
//! slot before any is revealed, and no border is drawn for it.

use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;

use super::{REFRESH, animated, placements, swap, two};

#[test]
fn stopping_ends_every_motion_in_its_slot_before_any_window_is_revealed() {
    for noted in [false, true] {
        // Arrange -- nothing posted to either window yet, and a display
        // change noted or not, which stops the motions where they are.
        let (platform, mut manager) = two(&animated());
        let (left, right) = (platform.window(1).rect, platform.window(2).rect);
        swap(&mut manager);
        if noted {
            manager.display_changing();
        }

        // Act
        manager.restore_all_windows();

        // Assert -- placed in full, and the borders the motions kept are
        // not drawn, as the tiler hides them all.
        let calls = platform.take_calls();
        assert_eq!(placements(&calls), [(2, left), (1, right)], "{noted}");
        let last_place = calls.iter().rposition(|c| matches!(c, Call::Place(..)));
        let first_reveal = calls.iter().position(|c| matches!(c, Call::Reveal(_)));
        assert!(last_place < first_reveal, "{calls:?}");
        let drawn = |call: &Call| matches!(call, Call::ShowBorder(..) | Call::MoveBorder(..));
        assert!(!calls.iter().any(drawn), "{calls:?}");
    }
}

#[test]
fn stopping_while_frames_are_on_their_way_puts_every_window_in_its_slot() {
    // Arrange -- a swap one frame in: window 1's application is slow to
    // handle its frames, window 2's handled its first at once.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    platform.take_calls();

    // Act -- then the applications get to what was posted to them.
    manager.restore_all_windows();
    let calls = platform.take_calls();
    for id in [1, 2] {
        platform.catch_up(id);
    }

    // Assert -- each is placed in its slot and revealed, and window 1's
    // placement lands after the frame on its way.
    assert_eq!(placements(&calls), [(2, left), (1, right)]);
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
    for id in [1, 2] {
        assert!(calls.contains(&Call::Reveal(wid(id))), "{calls:?}");
    }
}

#[test]
fn stopping_while_a_display_change_settles_puts_the_windows_it_stopped_in_their_slots() {
    // Arrange -- a swap three frames in when a display change is noted,
    // which stops both windows on their way.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    for _ in 0..3 {
        platform.advance(REFRESH);
        manager.frame();
    }
    manager.display_changing();
    platform.take_calls();

    // Act -- then the applications get to what was posted to them.
    manager.restore_all_windows();
    let calls = platform.take_calls();
    for id in [1, 2] {
        platform.catch_up(id);
    }

    // Assert
    assert_eq!(placements(&calls), [(2, left), (1, right)]);
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}
