//! What the log is told of the windows a display change stops: as they
//! stop, as they are placed at the targets they kept, after the change or
//! as the daemon stops, and as placements snap while the change settles.

use crate::{Action, Direction};

use super::super::{animated, land_prompt, step, swap, two};
use super::starting;

#[test]
fn a_display_change_is_logged_as_stopping_every_motion_and_placing_it_at_its_target() {
    // Arrange -- a swap, one frame on its way.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);

    // Act -- the change is noted, the monitors cannot be read, and the
    // windows get to their placements.
    manager.display_changing();
    let stopped = manager.transition_lines();
    manager.display_change_skipped();
    land_prompt(&platform, &mut manager);
    let placed = manager.transition_lines();

    // Assert
    for id in ["0x1", "0x2"] {
        let stop = format!(
            "transition: {id} stopped where it was, keeping its target, as the displays changed \
             after 16 ms of 180: "
        );
        assert_eq!(starting(&stopped, &stop), 1, "{stopped:#?}");
        let kept = format!(
            "transition: {id} placed at the target it kept, the display change over; its \
             placement posted behind what was on its way"
        );
        assert_eq!(starting(&placed, &kept), 1, "{placed:#?}");
        let landed = format!("transition: {id} is in place, ");
        assert_eq!(starting(&placed, &landed), 1, "{placed:#?}");
    }
}

#[test]
fn a_placement_while_the_displays_change_is_logged_as_snapping_for_them() {
    // Arrange
    let (_platform, mut manager) = two(&animated());
    manager.display_changing();

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();

    // Assert -- both windows of the swap snap, and none moves.
    let lines = manager.transition_lines();
    for id in ["0x1", "0x2"] {
        let snaps = format!("transition: {id} snaps: the displays are changing");
        assert_eq!(starting(&lines, &snaps), 1, "{lines:#?}");
    }
    assert_eq!(starting(&lines, "transition: 2 window(s) move"), 0);
}

#[test]
fn a_window_a_display_change_stopped_is_logged_as_placed_as_the_daemon_stops() {
    // Arrange -- a swap, one frame on its way, and a display change.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);
    manager.display_changing();
    manager.transition_lines();

    // Act
    manager.restore_all_windows();

    // Assert -- each placed at the target it kept, and nothing else ends.
    let lines = manager.transition_lines();
    for id in ["0x1", "0x2"] {
        let kept = format!("transition: {id} placed at the target it kept as the daemon stopped");
        assert_eq!(starting(&lines, &kept), 1, "{lines:#?}");
    }
    assert_eq!(lines.len(), 2, "{lines:#?}");
}
