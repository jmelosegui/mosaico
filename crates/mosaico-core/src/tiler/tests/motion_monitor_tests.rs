//! Transitions of a step that moves a window to another monitor.

use std::time::Duration;

use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::start;
use crate::{Action, Direction};

use super::{REFRESH, animated, frame_by_frame, posted, settle};

/// How many frames among `steps` were sent to window `id`.
fn frames_of(steps: &[Vec<super::Call>], id: u64) -> usize {
    let posted = steps.iter().flat_map(|calls| posted(calls)).flatten();
    posted.filter(|frame| frame.window.raw() == id).count()
}

#[test]
fn neighbours_glide_in_full_when_the_window_moved_to_another_monitor_is_slow_to_place() {
    // Arrange -- window 2 leaves monitor 0 for monitor 1, where window 3
    // is. It snaps there, and its application keeps the placement waiting
    // 150 ms as it repaints on its new monitor. Windows 1 and 3 glide to
    // make room for it.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &animated());
    manager.flush_pending();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    settle(&platform, &mut manager);
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
    platform.edit(2, |w| w.place_takes = Duration::from_millis(150));
    platform.take_calls();

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    let steps = frame_by_frame(&platform, &mut manager);

    // Assert -- their motions start once the step is placed, not before
    // the wait, which would leave them a frame or two: a jump.
    let duration = animated().animations.spec().duration;
    let full = (duration.as_millis() / REFRESH.as_millis()) as usize;
    for id in [1, 3] {
        let frames = frames_of(&steps, id);
        assert!(frames + 1 >= full, "window {id}: {frames} of {full} frames");
    }
}

/// Windows 1 and 2 on monitor 0 and window 3 on monitor 1, with window 2
/// focused, and each placement of it keeping the caller waiting `takes`.
fn slow_to_place(takes: Duration) -> (FakePlatform, super::Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &animated());
    manager.flush_pending();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    settle(&platform, &mut manager);
    platform.edit(2, |w| w.place_takes = takes);
    manager.transition_lines();
    platform.take_calls();
    (platform, manager)
}

#[test]
fn the_log_times_motions_started_late_from_when_they_started() {
    // Arrange
    let (platform, mut manager) = slow_to_place(Duration::from_millis(150));

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    frame_by_frame(&platform, &mut manager);

    // Assert -- each ends a frame step at most after its full duration,
    // not after the wait as well.
    let duration = animated().animations.spec().duration.as_millis();
    let lines = manager.transition_lines();
    for id in [1, 3] {
        let start = format!("transition: 0x{id} ended at its target after ");
        let line = lines.iter().find(|l| l.starts_with(&start));
        let line = line.unwrap_or_else(|| panic!("no end for window {id}: {lines:#?}"));
        let took: u128 = line[start.len()..]
            .split(' ')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let steps = duration..=duration + REFRESH.as_millis();
        assert!(steps.contains(&took), "{line}");
        assert!(line.contains(&format!("ms of {duration}:")), "{line}");
    }
}
