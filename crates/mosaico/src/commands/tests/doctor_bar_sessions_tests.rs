use mosaico_core::StableMonitorId;
use mosaico_core::ipc::{DaemonStatus, PROTOCOL_VERSION, Response};

use super::*;
use crate::commands::context::Exit;
use crate::commands::test_support::Harness;

/// `top` pixels kept on `monitor`.
fn top(monitor: MonitorTarget, top: u32) -> Reserve {
    Reserve {
        monitor,
        top,
        ..Reserve::default()
    }
}

/// Session `session`, open, with nothing reserved.
fn open(session: u64) -> BarSession {
    BarSession {
        session,
        may_reserve: true,
        ..BarSession::default()
    }
}

#[test]
fn each_session_says_who_opened_it_and_the_space_it_keeps() {
    // Arrange
    let yasb = BarSession {
        bar: Some("yasb".into()),
        client: Some("yasb 2.0.8".into()),
        process: Some(42),
        reserves: vec![
            top(MonitorTarget::All, 32),
            Reserve {
                monitor: MonitorTarget::One(StableMonitorId::new("DEL-1")),
                left: 8,
                bottom: 20,
                ..Reserve::default()
            },
        ],
        ..open(1)
    };
    let script = BarSession {
        client: Some("mosaico subscribe".into()),
        ..open(2)
    };
    let gone = BarSession {
        bar: Some("zebar".into()),
        ended: true,
        reserves: vec![top(MonitorTarget::All, 40)],
        ..open(3)
    };
    let bar = BarStatus {
        provider: "external".into(),
        sessions: vec![yasb, script, gone, open(4)],
    };

    // Act
    let lines = session_lines(&bar, "external");

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Bar API: 3 sessions open"),
            "       Session 1: bar \"yasb\" (yasb 2.0.8), process 42: reserves top 32 on every \
             monitor; bottom 20, left 8 on monitor DEL-1"
                .to_string(),
            "       Session 2: mosaico subscribe: reserves nothing".to_string(),
            "       Session 3: bar \"zebar\": ended; its space is kept a few seconds for a bar \
             that starts again: top 40 on every monitor"
                .to_string(),
            "       Session 4: a client that gave no name: reserves nothing".to_string(),
        ]
    );
}

#[test]
fn a_session_that_ended_is_listed_under_no_session_open() {
    // Arrange
    let gone = BarSession {
        client: Some("mosaico subscribe".into()),
        ended: true,
        reserves: vec![top(MonitorTarget::All, 40)],
        ..open(1)
    };
    let bar = BarStatus {
        provider: "external".into(),
        sessions: vec![gone],
    };

    // Act
    let lines = session_lines(&bar, "external");

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Bar API: no session open"),
            "       Session 1: mosaico subscribe: ended; its space is kept a few seconds for a \
             bar that starts again: top 40 on every monitor"
                .to_string(),
        ]
    );
}

#[test]
fn a_session_the_provider_lets_reserve_nothing_says_so() {
    // Arrange
    let bar = BarStatus {
        provider: "mosaico".into(),
        sessions: vec![BarSession {
            bar: Some("yasb".into()),
            may_reserve: false,
            ..open(7)
        }],
    };

    // Act
    let lines = session_lines(&bar, "mosaico");

    // Assert
    assert_eq!(
        lines[1],
        "       Session 7: bar \"yasb\": reserves nothing, and may not while the bar is \"mosaico\""
    );
}

#[test]
fn a_daemon_running_another_bar_than_the_file_picks_is_flagged() {
    // Arrange
    let bar = BarStatus {
        provider: "mosaico".into(),
        sessions: Vec::new(),
    };

    // Act
    let lines = session_lines(&bar, "external");

    // Assert
    assert_eq!(
        lines,
        [
            format!(
                "  {WARN} The daemon runs the bar \"mosaico\", not \"external\" as config.toml \
                 says: it takes the file in once it reads without errors"
            ),
            format!("  {OK} Bar API: no session open"),
        ]
    );
}

/// What `check_sessions` says when the daemon's status carries `bar`.
fn checked(test: &str, bar: Option<BarStatus>) -> String {
    let mut h = Harness::new(test);
    h.world.daemon_up(9);
    let status = DaemonStatus {
        version: "1.0.0".into(),
        protocol: PROTOCOL_VERSION,
        windows: 0,
        bar,
    };
    h.world.reply(Ok(Response::ok().with_data(&status)));
    let exit = h.run(|cx| {
        check_sessions(cx);
        Exit::Success
    });
    assert_eq!(exit, Exit::Success);
    h.out()
}

#[test]
fn doctor_lists_the_sessions_the_running_daemon_reports() {
    // Arrange
    let bar = BarStatus {
        provider: "mosaico".into(),
        sessions: vec![open(1)],
    };

    // Act
    let out = checked("doctor-sessions", Some(bar));

    // Assert
    assert!(out.contains("Bar API: 1 session open"), "{out}");
    assert!(
        out.contains("Session 1: a client that gave no name"),
        "{out}"
    );
}

#[test]
fn a_daemon_older_than_the_bar_api_gets_no_line() {
    // Act
    let out = checked("doctor-sessions-old", None);

    // Assert
    assert_eq!(out, "");
}
