//! Starting applications the user asked for by name.

use crate::Result;

/// Whether a started program shows its windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// It shows its windows as it normally would.
    Shown,
    /// It starts without a window: one that would open a console for a
    /// command-line program opens none, so nothing flashes on screen.
    Hidden,
}

/// Starts an application the way the system's own run box would.
///
/// # Contract for platform implementations
///
/// - `command` is what the user wrote: a name the system finds on its own,
///   such as one on the search path or an application registered with the
///   system, a full path, or a URI the system has a handler for.
///   Environment variables in `command` and `args` are expanded the way
///   the system's shell expands them.
/// - `args` reach the program as separate arguments, exactly as given,
///   even when one holds spaces or quotes. The platform quotes them as its
///   programs expect.
/// - The program starts in the user's home folder, detached from the
///   caller: it shares no console or open handles with it, keeps running
///   after the caller exits, and `launch` returns without waiting for it
///   to show a window.
/// - With [`Visibility::Hidden`], the program shows no window when it
///   starts. A command-line program gets no visible console, not even for
///   a moment, whichever terminal the system would otherwise open for it.
/// - Nothing is shown to the user when the program cannot be started, and
///   a name that is found nowhere is not guessed into something else, such
///   as a web address. `launch` fails with
///   [`Error::Platform`](crate::Error::Platform) instead, and the caller
///   reports it.
pub trait Launcher {
    /// Starts `command` with `args`, shown or hidden as `visibility` says.
    fn launch(&self, command: &str, args: &[String], visibility: Visibility) -> Result<()>;
}
