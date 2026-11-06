//! Launch bindings: starting the program a key is bound to.

use crate::Result;
use crate::config::LaunchBinding;
use crate::system::Launcher;

use super::{Platform, TilingManager};

impl<P: Platform> TilingManager<P> {
    /// Carries out a press of `binding`: starts its program, with its
    /// arguments, through `launcher`.
    ///
    /// The new window is taken in like any other once it is shown.
    ///
    /// # Errors
    ///
    /// When `launcher` cannot start the program.
    pub fn launch(&mut self, binding: &LaunchBinding, launcher: &dyn Launcher) -> Result<()> {
        launcher.launch(&binding.command, &binding.args)
    }
}
