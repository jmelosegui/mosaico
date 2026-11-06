//! Starting applications the user asked for by name.

use crate::Result;

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
/// - Nothing is shown to the user when the program cannot be started, and
///   a name that is found nowhere is not guessed into something else, such
///   as a web address. `launch` fails with
///   [`Error::Platform`](crate::Error::Platform) instead, and the caller
///   reports it.
pub trait Launcher {
    /// Starts `command` with `args`.
    fn launch(&self, command: &str, args: &[String]) -> Result<()>;
}
