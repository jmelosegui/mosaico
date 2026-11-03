//! Launch bindings carried out by the tiling manager.

use crate::config::{LaunchBinding, Modifier, WindowMatch};

use super::fake_launcher::RecordingLauncher;
use super::fake_platform::FakePlatform;
use super::manager_support::{start, still};

fn binding(command: &str, args: &[&str]) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: args.iter().map(ToString::to_string).collect(),
        key: "T".into(),
        modifiers: vec![Modifier::Alt],
        focus: false,
        windows: WindowMatch::default(),
    }
}

#[test]
fn a_launch_starts_the_program_with_its_arguments() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();
    let wt = binding("wt.exe", &["-p", "Ubuntu Dev"]);

    // Act
    let done = manager.launch(&wt, &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    let expected = ("wt.exe".to_string(), vec!["-p".into(), "Ubuntu Dev".into()]);
    assert_eq!(launcher.started(), [expected]);
}

#[test]
fn a_program_that_cannot_be_started_is_an_error() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::without("nope.exe");

    // Act
    let done = manager.launch(&binding("nope.exe", &[]), &launcher);

    // Assert
    assert!(done.is_err());
    assert!(launcher.started().is_empty());
}
