//! Stopping a command that runs until the user interrupts it.

use std::sync::mpsc::Sender;

use crate::Result;

/// Hearing about the user interrupting a command from the terminal, so
/// the command can stop cleanly instead of being ended.
///
/// # Contract for platform implementations
///
/// - Once `on_interrupt` succeeds, each interrupt from the terminal
///   (Ctrl+C) sends `()` on `tx` instead of ending the process. The
///   command decides when to stop.
/// - A receiver that went away is not an error; the interrupt is then
///   dropped.
/// - Callers set it up once per process. A second call fails with
///   [`Error::Platform`](crate::Error::Platform) and leaves the first
///   setup in place, so interrupts keep going to the first channel.
/// - When the platform cannot set it up at all, it fails with
///   [`Error::Platform`](crate::Error::Platform), and an interrupt ends the
///   process as it normally would.
/// - Once it is set up, a platform may also keep its other interrupts
///   from the terminal, such as a break key, from ending the process,
///   without sending anything for them. Its docs say which it keeps.
/// - A platform whose terminal cannot interrupt a process returns
///   [`Error::Unsupported`](crate::Error::Unsupported).
pub trait Interrupts {
    /// Sends `()` on `tx` for every interrupt from now on.
    fn on_interrupt(&self, tx: Sender<()>) -> Result<()>;
}
