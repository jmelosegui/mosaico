use mosaico_core::config::YASB;

use super::*;
use crate::commands::context::Exit;
use crate::commands::test_support::Harness;

/// Session `session` of the bar named `bar`, open unless `ended`.
fn session(session: u64, bar: &str, ended: bool) -> BarSession {
    BarSession {
        session,
        bar: Some(bar.into()),
        ended,
        may_reserve: true,
        ..BarSession::default()
    }
}

fn yasb_bar(sessions: Vec<BarSession>) -> BarStatus {
    BarStatus {
        provider: "yasb".into(),
        sessions,
    }
}

#[test]
fn a_program_that_is_not_running_says_how_to_start() {
    // Act
    let lines = program_lines(&YASB, &[], Some(&yasb_bar(Vec::new())));

    // Assert
    assert_eq!(
        lines,
        [format!(
            "  {WARN} YASB is not running: start it, then run `mosaico bar setup yasb`"
        )]
    );
}

#[test]
fn a_running_program_without_a_session_for_its_bar_is_only_said_to_run() {
    // Arrange
    let bar = yasb_bar(vec![session(1, "zebar", false), session(2, "yasb", true)]);

    // Act
    let lines = program_lines(&YASB, &[5320], Some(&bar));

    // Assert
    assert_eq!(lines, [format!("  {OK} YASB is running (process 5320)")]);
}

#[test]
fn a_running_program_with_open_sessions_for_its_bar_is_connected() {
    // Arrange
    let bar = yasb_bar(vec![session(3, "YASB", false), session(4, "yasb", false)]);

    // Act
    let lines = program_lines(&YASB, &[10, 11], Some(&bar));

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} YASB is running (processes 10, 11)"),
            format!("  {OK} YASB is connected to mosaico (sessions 3, 4)"),
        ]
    );
}

#[test]
fn without_an_answer_from_the_daemon_only_the_process_is_checked() {
    // Act
    let lines = program_lines(&YASB, &[5320], None);

    // Assert
    assert_eq!(lines, [format!("  {OK} YASB is running (process 5320)")]);
}

#[test]
fn doctor_finds_the_program_the_configured_bar_names() {
    // Arrange
    let mut h = Harness::new("doctor-program");
    h.write_config("[bar]\nprovider = \"yasb\"\n");
    h.world.runs(77, "explorer");
    h.world.runs(5320, "yasb");

    // Act
    h.run(|cx| {
        check_program(cx, None);
        Exit::Success
    });

    // Assert
    assert_eq!(h.out(), format!("  {OK} YASB is running (process 5320)\n"));
}

#[test]
fn doctor_says_nothing_for_a_bar_without_a_program() {
    // Arrange
    let mut h = Harness::new("doctor-program-external");
    h.write_config("[bar]\nprovider = \"external\"\n");
    h.world.runs(5320, "yasb");

    // Act
    h.run(|cx| {
        check_program(cx, None);
        Exit::Success
    });

    // Assert
    assert_eq!(h.out(), "");
}
