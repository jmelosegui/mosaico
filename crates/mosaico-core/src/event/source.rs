//! Where window events come from.

use std::sync::mpsc::Sender;

use crate::{Error, Result, WindowEvent};

/// Watches the windows and displays of the desktop and reports what
/// happens to them as [`WindowEvent`]s.
///
/// Each variant of [`WindowEvent`] says when it is reported. What holds
/// for all of them is below.
///
/// # Contract for platform implementations
///
/// - `start` returns once the source is watching: what happens from then
///   on is reported until the [`EventWatch`] is stopped. When the
///   platform cannot watch windows at all, `start` fails and sends
///   nothing. When it can watch windows but not the displays, `start`
///   still succeeds, and [`EventWatch::display_error`] says why
///   [`WindowEvent::DisplayChanged`] and [`WindowEvent::WorkAreaChanged`]
///   will not come.
/// - Once the watch has stopped, nothing more is sent and the source has
///   dropped `events`, so a loop over the receiver ends. A receiver that
///   went away is not an error: events are dropped until the watch
///   stops. `start` may be called again after a watch stopped.
/// - The windows of this process, such as the bars and the borders, are
///   never reported. What this process does to other windows is reported
///   like anything else: the tiler counts on its own minimize being
///   reported back, and recognizes the rest as its own.
/// - Events may name windows that will never be managed, such as windows
///   inside other windows, menus or the desktop, and any event may be
///   reported more than once.
/// - Events about one window are sent in the order the platform saw
///   them. Nothing else is promised across windows, or between window
///   and display events, with one exception: [`WindowEvent::Focused`]
///   reports are sent in the order the focus changed, whichever windows
///   they name, because the tiler follows the last one.
/// - [`WindowEvent::MouseHover`] is never sent. It comes from
///   [`FocusFollowsMouse`](super::FocusFollowsMouse).
/// - A kind of event the platform cannot observe is left out, never made
///   up, and the implementation's docs say which. Where a variant has a
///   fallback, its docs say what to report instead.
pub trait EventSource {
    /// Starts reporting to `events`, and returns once watching.
    fn start(&self, events: Sender<WindowEvent>) -> Result<EventWatch>;
}

/// A running [`EventSource`]. Dropping it stops the source, as
/// [`stop`](Self::stop) does.
#[must_use = "dropping the watch stops the events"]
pub struct EventWatch {
    stop: Option<Box<dyn FnOnce() + Send>>,
    display_error: Option<Error>,
}

impl EventWatch {
    /// Wraps how a platform stops its source.
    ///
    /// `stop` runs once, from [`stop`](Self::stop) or on drop. It must not
    /// return before the source has stopped sending and dropped its
    /// sender.
    pub fn new(stop: impl FnOnce() + Send + 'static) -> Self {
        Self {
            stop: Some(Box::new(stop)),
            display_error: None,
        }
    }

    /// The same watch, for a source that watches windows but could not
    /// watch the displays, because of `error`.
    pub fn without_displays(mut self, error: Error) -> Self {
        self.display_error = Some(error);
        self
    }

    /// Why display and work-area changes will not be reported, when the
    /// source could not watch the displays. Window events still are.
    pub fn display_error(&self) -> Option<&Error> {
        self.display_error.as_ref()
    }

    /// Stops the source and waits until it has stopped.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for EventWatch {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop();
        }
    }
}

#[cfg(test)]
#[path = "tests/source_tests.rs"]
mod tests;
