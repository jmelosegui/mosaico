//! Launch bindings carried out by the tiling manager.

use crate::Action;
use crate::config::{LaunchBinding, Modifier, WindowMatch};
use crate::id::wid;
use crate::tiler::Launched;

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

/// A binding for `command` that focuses an open window of it.
fn focusing(command: &str) -> LaunchBinding {
    LaunchBinding {
        focus: true,
        ..binding(command, &[])
    }
}

/// Makes window `id` one of the program at `/apps/edge`.
fn edge(platform: &FakePlatform, id: u64) {
    platform.edit(id, |w| w.facts.as_mut().unwrap().path = "/apps/edge".into());
}

#[test]
fn without_focus_the_program_starts_even_when_its_window_is_open() {
    // Arrange -- window 1 belongs to the program.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    edge(&platform, 1);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();

    // Act
    let done = manager.launch(&binding("edge", &[]), &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(launcher.started().len(), 1);
}

#[test]
fn with_focus_an_open_window_of_the_program_is_focused_instead() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    edge(&platform, 2);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();

    // Act
    let done = manager.launch(&focusing("edge"), &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert!(launcher.started().is_empty());
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
}

#[test]
fn with_focus_the_program_starts_when_none_of_its_windows_is_open() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();

    // Act
    let done = manager.launch(&focusing("edge"), &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(launcher.started().len(), 1);
}

#[test]
fn pressing_again_cycles_through_the_windows_of_the_program() {
    // Arrange -- windows 1 and 3 belong to the program, 2 does not.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    edge(&platform, 1);
    edge(&platform, 3);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();
    let key = focusing("edge");

    // Act
    let mut focused = Vec::new();
    for _ in 0..3 {
        manager.launch(&key, &launcher).unwrap();
        focused.push(manager.tiler().focused_window().unwrap());
    }

    // Assert -- each press moves on to the other window, never to 2.
    assert!(launcher.started().is_empty());
    assert_ne!(focused[0], focused[1]);
    assert_eq!(focused[0], focused[2]);
    assert!(focused.iter().all(|w| [wid(1), wid(3)].contains(w)));
}

#[test]
fn a_window_on_a_hidden_workspace_is_brought_into_view() {
    // Arrange -- window 1 belongs to the program and is sent to the
    // second workspace, then the monitor goes back to the first.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    edge(&platform, 1);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();
    let key = focusing("edge");
    manager.launch(&key, &launcher).unwrap();
    manager.handle_action(&Action::SendToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    assert_eq!(manager.tiler().monitors()[0].active_workspace(), 0);

    // Act
    let done = manager.launch(&key, &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(manager.tiler().monitors()[0].active_workspace(), 1);
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
    assert!(platform.window(1).visible && !platform.window(1).cloaked);
}

#[test]
fn a_window_behind_a_monocle_window_is_brought_in_front() {
    // Arrange -- window 2 fills the monitor in monocle, and window 1
    // belongs to the program.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    edge(&platform, 1);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();
    manager.launch(&focusing("app"), &launcher).unwrap();
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
    manager.handle_action(&Action::ToggleMonocle);

    // Act
    let done = manager.launch(&focusing("edge"), &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
    assert!(!manager.tiler().monitors()[0].active().monocle());
}

#[test]
fn a_window_on_another_monitor_is_focused_there() {
    // Arrange -- window 2, on the second monitor, belongs to the program.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    edge(&platform, 2);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();
    manager.launch(&focusing("app"), &launcher).unwrap();
    assert_eq!(manager.tiler().focused_monitor(), 0);

    // Act
    let done = manager.launch(&focusing("edge"), &launcher);

    // Assert
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(manager.tiler().focused_monitor(), 1);
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
}

#[test]
fn a_press_says_whether_it_focused_a_window_or_started_the_program() {
    // Arrange -- the daemon logged "started" for a press that only
    // focused an open window. Window 2 belongs to the program.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    edge(&platform, 2);
    let mut manager = start(&platform, None, &still());
    let launcher = RecordingLauncher::default();

    // Act
    let focused = manager.launch(&focusing("edge"), &launcher);
    let started = manager.launch(&focusing("wt"), &launcher);

    // Assert
    assert_eq!(focused.unwrap(), Launched::Focused);
    assert_eq!(started.unwrap(), Launched::Started);
}
