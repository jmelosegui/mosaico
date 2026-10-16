use super::*;

#[test]
fn every_ending_reads_as_what_became_of_the_motion() {
    // Arrange
    let stopped = |stop| Ending::Stopped(stop);
    let endings = [
        (Ending::AtTarget, "ended at its target"),
        (stopped(Stop::Hidden), "stopped where it was, hidden"),
        (stopped(Stop::Minimized), "stopped where it was, minimized"),
        (stopped(Stop::Maximized), "stopped where it was, maximized"),
        (
            stopped(Stop::Grabbed),
            "stopped where it was, grabbed by the user",
        ),
        (
            Ending::DisplaysChanged,
            "stopped where it was, keeping its target, as the displays changed",
        ),
        (Ending::Destroyed, "ended, its window gone"),
        (
            Ending::FellBehind,
            "went straight to its target, its step fell behind",
        ),
        (Ending::NewTarget, "ended with a placement that snaps"),
        (
            Ending::LetGo,
            "went straight to its target, let go of by the tiler",
        ),
        (Ending::Revealed, "went straight to its target, revealed"),
        (
            Ending::Shutdown,
            "went straight to its target as the daemon stopped",
        ),
        (
            Ending::FramesFailed,
            "went straight to its target, its frames failed",
        ),
        (
            Ending::TransitionsOff,
            "went straight to its target, transitions turned off",
        ),
    ];

    // Act / Assert
    for (ending, text) in endings {
        assert_eq!(ending.to_string(), text);
    }
}
