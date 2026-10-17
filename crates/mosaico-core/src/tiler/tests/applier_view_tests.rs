//! The desktop as the tiler reads it while changes of show state are on
//! their way to its windows.

use std::collections::HashMap;

use crate::id::wid;
use crate::tiler::Platform;
use crate::tiler::tests::fake_platform::FakePlatform;

use super::super::shows::ShowState;
use super::*;

#[test]
fn a_window_a_hide_or_a_minimize_is_on_its_way_to_is_not_on_screen() {
    // Arrange -- three windows the desktop still has on screen.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let states = HashMap::from([
        (wid(1), ShowState::Hidden),
        (wid(2), ShowState::Minimized),
        (wid(3), ShowState::Shown),
    ]);

    // Act
    let view = PostedView::new(platform.desktop(), platform.control(), &states);

    // Assert
    assert_eq!(view.windows_on_screen(), [wid(3)]);
}
