//! Forgetting the windows that no longer need their changes posted once
//! transitions are off: past the grace, with an application that
//! responds.

use super::*;

/// Whether a change made to window 1 now is posted behind.
fn posts_behind(table: &mut FrameTable<i32>, probe: &Probe) -> bool {
    table.posts_behind(wid(1), probe, probe.now)
}

#[test]
fn settling_forgets_a_window_past_the_grace_whose_application_responds() {
    // Arrange -- a placement posted after the frame, which holds until
    // the window moves.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));
    table.posted(wid(1), &probe, probe.now);
    probe.now += ASYNC_GRACE;

    // Act
    table.settle_responding(&probe, probe.now);

    // Assert -- as one never given a frame.
    assert!(!posts_behind(&mut table, &probe));
    assert_eq!(table.kept_for(wid(1), &probe), None);
}

#[test]
fn settling_keeps_a_window_within_the_grace_or_whose_application_hangs() {
    // Arrange -- something posted to window 1 just now, and window 2
    // given a frame while it hangs, long ago.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    let hung = Window {
        owner: 2,
        responding: false,
        ..*probe.window()
    };
    probe.windows.insert(wid(2), hung);
    let late = Frame {
        window: wid(2),
        ..frame(120, true)
    };
    place(&mut table, &mut probe, late);
    probe.now += ASYNC_GRACE * 2;
    table.posted(wid(1), &probe, probe.now);

    // Act
    table.settle_responding(&probe, probe.now);

    // Assert
    assert!(posts_behind(&mut table, &probe));
    assert!(table.posts_behind(wid(2), &probe, probe.now));
}
