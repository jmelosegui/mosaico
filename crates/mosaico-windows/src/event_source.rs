//! Window events from Win32: two event hooks, and a hidden window for
//! what Windows broadcasts about displays and the work area.
//!
//! | Win32 report | Event |
//! | --- | --- |
//! | `EVENT_OBJECT_CREATE`, `EVENT_OBJECT_SHOW` | `Created` |
//! | `EVENT_OBJECT_DESTROY` | `Destroyed` |
//! | `EVENT_OBJECT_HIDE` | `Hidden` |
//! | `EVENT_SYSTEM_FOREGROUND` | `Focused` |
//! | `EVENT_SYSTEM_MOVESIZESTART` | `MoveStarted` |
//! | `EVENT_SYSTEM_MOVESIZEEND` | `Moved` |
//! | `EVENT_SYSTEM_MINIMIZESTART` / `MINIMIZEEND` | `Minimized` / `Restored` |
//! | `EVENT_OBJECT_NAMECHANGE` | `TitleChanged` |
//! | `EVENT_OBJECT_LOCATIONCHANGE` | `LocationChanged` |
//! | `WM_DISPLAYCHANGE` | `DisplayChanged` |
//! | `WM_SETTINGCHANGE` for `SPI_SETWORKAREA` | `WorkAreaChanged` |
//!
//! How the Win32 reports meet the core contract:
//!
//! - `WINEVENT_SKIPOWNPROCESS` keeps the bars and borders out.
//! - `WINEVENT_OUTOFCONTEXT` reports arrive in the order they were made,
//!   and both hooks deliver on the one event thread, so every event keeps
//!   its order, focus changes included.
//! - Only reports on the window object itself (`OBJID_WINDOW`) are kept,
//!   except creation, show and name changes: WPF windows, such as Visual
//!   Studio's, make those only on an object inside the window.
//! - Windows hides a window as it destroys it, so a window that closes
//!   while shown reports `EVENT_OBJECT_HIDE` before
//!   `EVENT_OBJECT_DESTROY`, which become `Hidden` and then `Destroyed`.
//! - GPU-accelerated applications (Zed, browsers) report
//!   `EVENT_OBJECT_HIDE` for their top-level window while they recreate
//!   their rendering surface, and stay visible. It becomes `Hidden`,
//!   which the tiler checks against the window's visibility.
//! - The mouse pointer reports being shown and changing shape with no
//!   window, and those reports are dropped.
//! - `EVENT_SYSTEM_MOVESIZESTART` starts a drag the user makes, and
//!   `EVENT_SYSTEM_MOVESIZEEND` ends it, even one cancelled with Escape.
//!   Maximizing and restoring from the title bar do not drag, so they
//!   only report `EVENT_OBJECT_LOCATIONCHANGE`.
//! - When the hidden window cannot be created, the source still runs
//!   without display and work-area changes, and `EventWatch::display_error`
//!   says why.

use std::cell::RefCell;
use std::sync::mpsc::Sender;

use mosaico_core::event::{EventSource, EventWatch};
use mosaico_core::{Error, Result, WindowEvent};

use crate::message_thread::{self, MessageThread};

#[path = "event_source_hooks.rs"]
mod hooks;
#[path = "event_source_sink.rs"]
mod sink;

// Where the callbacks on the event thread send, since Win32 calls them
// without any context of ours.
thread_local! {
    static EVENTS: RefCell<Option<Sender<WindowEvent>>> = const { RefCell::new(None) };
}

/// Sends an event from a callback on the event thread. A receiver that
/// went away is not an error.
fn send(event: WindowEvent) {
    EVENTS.with(|events| {
        if let Some(events) = events.borrow().as_ref() {
            let _ = events.send(event);
        }
    });
}

/// The Win32 [`EventSource`].
///
/// The hooks and the hidden window deliver to the thread that set them
/// up, so they live on a thread of their own, which pumps messages for
/// them until the watch is stopped.
pub struct WindowsEventSource;

impl EventSource for WindowsEventSource {
    fn start(&self, events: Sender<WindowEvent>) -> Result<EventWatch> {
        watch(events, || set_up(hooks::Hooks::install, sink::Sink::create))
    }
}

/// What the event thread owns while it watches, and why it does not
/// watch the displays, when it does not.
struct SetUp<T> {
    owned: T,
    display_error: Option<Error>,
}

/// Runs `set_up` on a new event thread whose callbacks send to `events`,
/// and returns once it has. The thread then pumps messages until the
/// watch is stopped, and drops what `set_up` returned before it ends. The
/// display error goes to the watch.
fn watch<T>(
    events: Sender<WindowEvent>,
    set_up: impl FnOnce() -> Result<SetUp<T>> + Send + 'static,
) -> Result<EventWatch> {
    let (thread, display_error) = MessageThread::spawn("window events", move |ready| {
        EVENTS.with(|cell| *cell.borrow_mut() = Some(events));
        match set_up() {
            Ok(set_up) => {
                ready.report(Ok(set_up.display_error));
                message_thread::pump();
                drop(set_up.owned);
            }
            Err(e) => ready.report(Err(e)),
        }
        EVENTS.with(|cell| cell.borrow_mut().take());
    })?;
    let watching = EventWatch::new(move || thread.stop());
    Ok(match display_error {
        Some(e) => watching.without_displays(e),
        None => watching,
    })
}

/// Installs the hooks, then creates the hidden window. They are owned
/// window first, so that dropping them destroys the window before the
/// hooks are removed.
fn set_up<H, S>(
    hooks: impl FnOnce() -> Result<H>,
    sink: impl FnOnce() -> Result<S>,
) -> Result<SetUp<(Option<S>, H)>> {
    let hooks = hooks()?;
    // Without the window the hooks still work, and only display and
    // work-area changes are missed, so the caller is told why instead of
    // the start failing.
    let (sink, display_error) = match sink() {
        Ok(sink) => (Some(sink), None),
        Err(e) => (None, Some(e)),
    };
    Ok(SetUp {
        owned: (sink, hooks),
        display_error,
    })
}

#[cfg(test)]
#[path = "event_source_tests.rs"]
mod tests;
