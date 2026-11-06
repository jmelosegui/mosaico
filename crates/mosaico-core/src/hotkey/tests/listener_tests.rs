use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::config::{Keybinding, LaunchBinding, WindowMatch};
use crate::hotkey::{HotkeyControl, HotkeyListener, HotkeySource, HotkeyWatch, Target};
use crate::{Error, Result};

/// What the sources were asked to do, oldest first.
type Calls = Arc<Mutex<Vec<String>>>;

/// A source that records each start with the commands it was given, and
/// what its watches are asked, and fails to start when told to.
#[derive(Default)]
struct Recording {
    calls: Calls,
    fails: Arc<Mutex<bool>>,
}

struct Control(Calls);

impl HotkeyControl for Control {
    fn pause(&self) -> Result<()> {
        self.0.lock().unwrap().push("pause".into());
        Ok(())
    }

    fn unpause(&self) -> Result<()> {
        self.0.lock().unwrap().push("unpause".into());
        Ok(())
    }

    fn stop(self: Box<Self>) {
        self.0.lock().unwrap().push("stop".into());
    }
}

impl HotkeySource for Recording {
    fn start(
        &self,
        _bindings: Vec<Keybinding>,
        launches: Vec<LaunchBinding>,
        _presses: mpsc::Sender<Target>,
    ) -> Result<HotkeyWatch> {
        if *self.fails.lock().unwrap() {
            return Err(Error::platform("watch hotkeys", "no thread"));
        }
        let commands: Vec<_> = launches.iter().map(|l| l.command.as_str()).collect();
        let call = format!("start [{}]", commands.join(", "));
        self.calls.lock().unwrap().push(call);
        Ok(HotkeyWatch::new(Control(Arc::clone(&self.calls))))
    }
}

fn app(command: &str) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: Vec::new(),
        key: "T".into(),
        modifiers: Vec::new(),
        focus: false,
        windows: WindowMatch::default(),
    }
}

/// A listener started with `launches`, and what its sources record.
fn listening(launches: &[&str]) -> (HotkeyListener<Recording>, Calls, Arc<Mutex<bool>>) {
    let source = Recording::default();
    let (calls, fails) = (Arc::clone(&source.calls), Arc::clone(&source.fails));
    let (presses, _) = mpsc::channel();
    let launches = launches.iter().map(|c| app(c)).collect();
    let listener = HotkeyListener::start(source, Vec::new(), launches, presses).unwrap();
    (listener, calls, fails)
}

fn calls(calls: &Calls) -> Vec<String> {
    calls.lock().unwrap().clone()
}

#[test]
fn a_restart_stops_the_old_hotkeys_before_it_starts_the_new_ones() {
    // Arrange
    let (mut listener, recorded, _) = listening(&["notepad"]);

    // Act
    let restarted = listener.restart(Vec::new(), vec![app("notepad"), app("wt")]);

    // Assert
    assert!(restarted.is_ok());
    assert_eq!(
        calls(&recorded),
        ["start [notepad]", "stop", "start [notepad, wt]"]
    );
}

#[test]
fn a_launch_press_counts_in_the_launch_bindings_of_the_last_restart() {
    // Arrange
    let (mut listener, _, _) = listening(&["notepad"]);

    // Act
    listener
        .restart(Vec::new(), vec![app("wt"), app("notepad")])
        .unwrap();

    // Assert
    assert_eq!(listener.launch(0).map(|l| l.command.as_str()), Some("wt"));
    assert_eq!(listener.launch(2), None);
}

#[test]
fn hotkeys_paused_before_a_restart_stay_paused_after_it() {
    // Arrange
    let (mut listener, recorded, _) = listening(&[]);
    listener.pause().unwrap();

    // Act
    listener.restart(Vec::new(), vec![app("wt")]).unwrap();

    // Assert
    assert_eq!(
        calls(&recorded),
        ["start []", "pause", "stop", "start [wt]", "pause"]
    );
}

#[test]
fn hotkeys_unpaused_before_a_restart_are_not_paused_by_it() {
    // Arrange
    let (mut listener, recorded, _) = listening(&[]);
    listener.pause().unwrap();
    listener.unpause().unwrap();

    // Act
    listener.restart(Vec::new(), Vec::new()).unwrap();

    // Assert
    assert_eq!(
        calls(&recorded),
        ["start []", "pause", "unpause", "stop", "start []"]
    );
}

#[test]
fn a_restart_that_cannot_start_leaves_no_hotkeys_until_one_that_can() {
    // Arrange
    let (mut listener, recorded, fails) = listening(&["notepad"]);
    *fails.lock().unwrap() = true;

    // Act
    let failed = listener.restart(Vec::new(), vec![app("wt")]);
    *fails.lock().unwrap() = false;
    let paused = listener.pause();
    let retried = listener.restart(Vec::new(), vec![app("wt")]);

    // Assert
    assert!(matches!(failed, Err(Error::Platform { .. })));
    assert!(paused.is_ok() && retried.is_ok());
    assert_eq!(
        calls(&recorded),
        ["start [notepad]", "stop", "start [wt]", "pause"]
    );
}

#[test]
fn stopping_the_listener_stops_its_hotkeys() {
    // Arrange
    let (listener, recorded, _) = listening(&[]);

    // Act
    listener.stop();

    // Assert
    assert_eq!(calls(&recorded), ["start []", "stop"]);
}
