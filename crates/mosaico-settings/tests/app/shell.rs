//! The window's frame: search and the daemon status.

use crate::support::*;

#[test]
fn search_shows_only_the_settings_that_match() {
    let mut h = stopped();

    search(&mut h, "rotated");
    h.run();

    assert!(
        h.query_by_role_and_label(Role::SpinButton, "Maximum file size")
            .is_some()
    );
    assert_eq!(h.query_all_by_label("Theme").count(), 0);
    assert_eq!(h.query_all_by_label("Startup").count(), 0);
}

#[test]
fn search_says_when_nothing_matches() {
    let mut h = stopped();

    search(&mut h, "zzz");
    h.run();

    h.get_by_label("No settings match \u{201c}zzz\u{201d}.");
}

#[test]
fn shows_whether_the_daemon_runs() {
    stopped().get_by_label("Daemon stopped");

    let running = harness(DaemonState::Running {
        version: "9.9.9".into(),
    });
    running.get_by_label("Daemon running");
    running.get_by_label("mosaico 9.9.9");
}
