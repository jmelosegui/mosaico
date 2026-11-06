use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use crate::config::{LaunchBinding, WindowMatch};
use crate::tiler::Launched;
use crate::tray::{TrayCommand, TrayControl, TrayNotice, TraySource, TrayState, TrayWatch};
use crate::{Error, Result};

use super::*;

/// What the source was asked to do, oldest first.
type Calls = Arc<Mutex<Vec<String>>>;

/// A source that records what it is asked, and can fail to start.
#[derive(Clone, Default)]
struct Recording {
    calls: Calls,
    fails: bool,
}

struct Control(Calls);

impl Recording {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl TraySource for Recording {
    fn start(&self, state: &TrayState, _: Sender<TrayCommand>) -> Result<TrayWatch> {
        if self.fails {
            return Err(Error::platform("show the tray icon", "no shell"));
        }
        let call = format!("start {}", state.tooltip());
        self.calls.lock().unwrap().push(call);
        Ok(TrayWatch::new(Control(Arc::clone(&self.calls))))
    }
}

impl TrayControl for Control {
    fn update(&self, state: &TrayState) -> Result<()> {
        let call = format!("update {}", state.tooltip());
        self.0.lock().unwrap().push(call);
        Ok(())
    }

    fn notify(&self, notice: &TrayNotice) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .push(format!("notify {}", notice.title));
        Ok(())
    }

    fn stop(self: Box<Self>) {
        self.0.lock().unwrap().push("stop".into());
    }
}

fn tray(source: &Recording) -> Tray {
    let (commands, _) = mpsc::channel();
    Tray::new(source.clone(), commands, "1.0.0")
}

#[test]
fn the_icon_is_not_shown_until_it_is_enabled() {
    // Arrange
    let source = Recording::default();
    let mut tray = tray(&source);

    // Act
    let before = tray.is_shown();
    tray.set_enabled(true);
    tray.set_enabled(true);

    // Assert -- enabling twice shows one icon.
    assert!(!before);
    assert!(tray.is_shown());
    assert_eq!(source.calls(), ["start mosaico 1.0.0"]);
}

#[test]
fn a_pause_while_hidden_shows_when_the_icon_comes_back() {
    // Arrange
    let source = Recording::default();
    let mut tray = tray(&source);
    tray.set_enabled(true);

    // Act
    tray.set_enabled(false);
    tray.set_paused(true);
    tray.notify(&TrayNotice::new("lost", ""));
    tray.set_enabled(true);

    // Assert -- nothing reaches a removed icon.
    assert_eq!(
        source.calls(),
        [
            "start mosaico 1.0.0",
            "stop",
            "start mosaico 1.0.0 (hotkeys paused)"
        ]
    );
}

#[test]
fn a_shown_icon_follows_the_pause_and_shows_notices() {
    // Arrange
    let source = Recording::default();
    let mut tray = tray(&source);
    tray.set_enabled(true);

    // Act -- the second pause changes nothing.
    tray.set_paused(true);
    tray.set_paused(true);
    tray.notify(&TrayNotice::new("bar.toml was not loaded", "why"));
    tray.set_paused(false);
    tray.stop();

    // Assert
    assert_eq!(
        source.calls(),
        [
            "start mosaico 1.0.0",
            "update mosaico 1.0.0 (hotkeys paused)",
            "notify bar.toml was not loaded",
            "update mosaico 1.0.0",
            "stop",
        ]
    );
}

#[test]
fn an_icon_that_cannot_be_shown_leaves_the_daemon_without_one() {
    // Arrange
    let source = Recording {
        fails: true,
        ..Recording::default()
    };
    let mut tray = tray(&source);

    // Act
    tray.set_enabled(true);
    tray.set_paused(true);

    // Assert
    assert!(!tray.is_shown());
    assert!(source.calls().is_empty());
}

fn launch(command: &str) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: Vec::new(),
        key: "T".into(),
        modifiers: Vec::new(),
        focus: false,
        windows: WindowMatch::default(),
    }
}

#[test]
fn a_program_a_launch_key_cannot_start_is_told_beside_the_icon() {
    // Arrange -- #207: a launch key for a program that does not exist did
    // nothing, and nothing said why.
    let source = Recording::default();
    let mut tray = tray(&source);
    tray.set_enabled(true);
    let missing = launch("no-such-app");
    let reason = "no program, file or URI named \"no-such-app\" was found";
    let failed = Err(Error::platform("start an application", reason));

    // Act -- a program that starts, or a window focused, says nothing.
    tray.report_launch(&launch("wt"), &Ok(Launched::Started));
    tray.report_launch(&launch("wt"), &Ok(Launched::Focused));
    tray.report_launch(&missing, &failed);

    // Assert
    assert_eq!(
        source.calls(),
        ["start mosaico 1.0.0", "notify no-such-app was not started"]
    );
}
