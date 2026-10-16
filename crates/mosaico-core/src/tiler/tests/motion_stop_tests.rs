//! Motions the tiler stopping ends: every window is placed in its slot
//! before any is revealed, the ones a display change stopped too.

use crate::tiler::tests::fake_platform::Call;

use super::{animated, placements, swap, two};

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

        // Assert -- placed in full.
        let calls = platform.take_calls();
        assert_eq!(placements(&calls), [(2, left), (1, right)], "{noted}");
        let last_place = calls.iter().rposition(|c| matches!(c, Call::Place(..)));
        let first_reveal = calls.iter().position(|c| matches!(c, Call::Reveal(_)));
        assert!(last_place < first_reveal, "{calls:?}");
    }
}
