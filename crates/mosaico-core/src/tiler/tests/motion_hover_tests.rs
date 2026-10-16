//! A pointer at rest while windows slide under it.

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;

use super::{Manager, animated, settle, swap, two};

/// Reports the pointer resting on window 2, and lets input go quiet.
fn hover_over_2(manager: &mut Manager) {
    manager.handle_event(&WindowEvent::MouseHover { window: wid(2) });
    manager.flush_pending();
}

#[test]
fn the_pointer_resting_on_a_window_does_not_focus_it_while_windows_move() {
    // Arrange -- window 1 has focus, and both are on their way.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    platform.take_calls();

    // Act
    hover_over_2(&mut manager);
    let while_moving = platform.take_calls();
    settle(&platform, &mut manager);
    platform.take_calls();
    hover_over_2(&mut manager);

    // Assert -- once they are in place, it does.
    assert!(
        !while_moving.contains(&Call::Focus(wid(2))),
        "{while_moving:?}"
    );
    assert!(platform.take_calls().contains(&Call::Focus(wid(2))));
}
