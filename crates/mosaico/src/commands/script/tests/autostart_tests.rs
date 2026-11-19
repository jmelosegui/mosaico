use std::fs;
use std::path::PathBuf;

use mosaico_core::system::LaunchCommand;

use crate::commands::context::Exit;
use crate::commands::script::run::{Run, execute};
use crate::commands::script::{autostart, files};
use crate::commands::test_support::{EXE, Harness};

fn script(h: &Harness, name: &str, source: &str) {
    let dir = h.config.dir().join("scripts");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{name}.lua")), source).unwrap();
}

#[test]
fn background_starts_the_script_again_detached_with_its_log() {
    // Arrange
    let mut h = Harness::new("script-background");
    script(&h, "dock", "");
    let args = vec!["n=1".to_string()];
    let run = Run {
        target: "dock",
        args: &args,
        fail_fast: true,
        background: true,
        log: None,
    };

    // Act
    let exit = h.run(|cx| execute(cx, &run));

    // Assert
    assert_eq!(exit, Exit::Success);
    let log = h.config.dir().join("logs").join("scripts").join("dock.log");
    assert_eq!(
        *h.world.relaunched.borrow(),
        vec![vec![
            "script".to_string(),
            "run".into(),
            "dock".into(),
            "n=1".into(),
            "--fail-fast".into(),
            "--log".into(),
            log.to_string_lossy().into_owned(),
        ]]
    );
    assert!(
        h.out().starts_with("Started dock in the background."),
        "{}",
        h.out()
    );
}

#[test]
fn a_logged_script_writes_its_output_and_errors_to_the_file() {
    // Arrange
    let mut h = Harness::new("script-log");
    script(&h, "noisy", "print('hello') error('then this')");
    let log: PathBuf = h
        .config
        .dir()
        .join("logs")
        .join("scripts")
        .join("noisy.log");
    let run = Run {
        target: "noisy",
        args: &[],
        fail_fast: false,
        background: false,
        log: Some(&log),
    };

    // Act
    let exit = h.run(|cx| execute(cx, &run));

    // Assert
    assert_eq!(exit, Exit::Failure);
    let written = fs::read_to_string(&log).unwrap();
    assert!(written.starts_with("hello\nError: "), "{written}");
    assert!(written.contains("then this"), "{written}");
    assert_eq!(h.scripts.printed(), "");
}

#[test]
fn autostart_registers_the_script_to_run_in_the_background() {
    // Arrange
    let mut h = Harness::new("script-autostart");
    script(&h, "dock", "");

    // Act
    let enabled = h.run(|cx| autostart::enable(cx, "dock"));
    let listed = h.run(files::list);

    // Assert
    assert_eq!((enabled, listed), (Exit::Success, Exit::Success));
    assert_eq!(
        h.world.script_logins.borrow().get("dock"),
        Some(&LaunchCommand::script(EXE, "dock"))
    );
    assert!(
        h.out().ends_with("dock (starts when you sign in)\n"),
        "{}",
        h.out()
    );
}

#[test]
fn autostart_refuses_a_script_that_is_not_there_and_disable_removes_it() {
    // Arrange
    let mut h = Harness::new("script-autostart-off");
    script(&h, "dock", "");
    h.run(|cx| autostart::enable(cx, "dock"));

    // Act
    let missing = h.run(|cx| autostart::enable(cx, "nope"));
    let disabled = h.run(|cx| autostart::disable(cx, "dock"));

    // Assert
    assert_eq!((missing, disabled), (Exit::Failure, Exit::Success));
    assert!(h.world.script_logins.borrow().is_empty());
}

#[test]
fn the_login_command_runs_the_script_in_the_background() {
    // Act
    let command = LaunchCommand::script(EXE, "dock");

    // Assert
    assert_eq!(
        command.to_string(),
        format!("\"{EXE}\" script run dock --background")
    );
}
