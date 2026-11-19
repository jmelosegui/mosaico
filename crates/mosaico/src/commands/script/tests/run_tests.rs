use std::fs;
use std::path::PathBuf;

use mosaico_core::MonitorId;
use mosaico_core::desktop_events::DesktopEvent;

use crate::commands::context::Exit;
use crate::commands::script::run::{Run, execute};
use crate::commands::test_support::Harness;

/// Writes `scripts\<name>.lua` in the harness's config folder.
fn script(h: &Harness, name: &str, source: &str) -> PathBuf {
    let dir = h.config.dir().join("scripts");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.lua"));
    fs::write(&path, source).unwrap();
    path
}

fn switched(workspace: usize) -> DesktopEvent {
    DesktopEvent::WorkspaceChanged {
        monitor: MonitorId::from_raw(1),
        workspace,
    }
}

#[test]
fn runs_a_script_by_its_name_with_its_arguments() {
    // Arrange
    let mut h = Harness::new("script-run");
    script(
        &h,
        "hello",
        "print('hello ' .. mosaico.args.who, mosaico.args.n + 1)",
    );

    // Act
    let exit = h.run(|cx| execute(cx, &plain("hello", &["who=you".into(), "n=1".into()])));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.scripts.printed(), "hello you\t2\n");
}

#[test]
fn runs_a_script_by_its_path() {
    // Arrange
    let mut h = Harness::new("script-run-path");
    let path = script(&h, "elsewhere", "print('ran')");
    let target = path.to_string_lossy().into_owned();

    // Act
    let exit = h.run(|cx| execute(cx, &plain(&target, &[])));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.scripts.printed(), "ran\n");
}

#[test]
fn a_missing_script_or_a_bad_argument_fails_with_why() {
    // Arrange
    let mut h = Harness::new("script-run-missing");
    script(&h, "ok", "");

    // Act
    let missing = h.run(|cx| execute(cx, &plain("nope", &[])));
    let bad = h.run(|cx| execute(cx, &plain("ok", &["flag".into()])));

    // Assert
    assert_eq!((missing, bad), (Exit::Failure, Exit::Failure));
    let err = h.err();
    assert!(err.contains("mosaico script list"), "{err}");
    assert!(err.contains("`flag` is not a key=value argument"), "{err}");
}

#[test]
fn an_error_in_the_script_fails_with_its_message() {
    // Arrange
    let mut h = Harness::new("script-run-error");
    script(&h, "broken", "error('it broke')");

    // Act
    let exit = h.run(|cx| execute(cx, &plain("broken", &[])));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("it broke"), "{}", h.err());
}

#[test]
fn stop_with_a_code_other_than_zero_fails() {
    // Arrange
    let mut h = Harness::new("script-run-stop");
    script(&h, "zero", "mosaico.stop()");
    script(&h, "two", "mosaico.stop(2)");

    // Act
    let zero = h.run(|cx| execute(cx, &plain("zero", &[])));
    let two = h.run(|cx| execute(cx, &plain("two", &[])));

    // Assert
    assert_eq!((zero, two), (Exit::Success, Exit::Failure));
}

#[test]
fn a_script_with_handlers_waits_for_the_daemon_to_come_back() {
    // Arrange -- the first subscription ends after one event, as when
    // the daemon stops; the second one stops the script.
    let mut h = Harness::new("script-run-wait");
    h.world.daemon_up(1);
    let queued = &h.scripts.daemon.subscriptions;
    queued.borrow_mut().push_back(vec![switched(0)]);
    queued.borrow_mut().push_back(vec![switched(4)]);
    script(
        &h,
        "watch",
        r#"
        mosaico.on("workspace_changed", function(e)
            print(e.workspace)
            if e.workspace == 5 then mosaico.stop() end
        end)
        "#,
    );

    // Act
    let exit = h.run(|cx| execute(cx, &plain("watch", &[])));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.scripts.printed(), "1\n5\n");
    assert_eq!(h.out(), "Waiting for mosaico to start...\n");
}

/// A run in the foreground with nothing but `target` and `args`.
fn plain<'a>(target: &'a str, args: &'a [String]) -> Run<'a> {
    Run {
        target,
        args,
        fail_fast: false,
        background: false,
        log: None,
    }
}
