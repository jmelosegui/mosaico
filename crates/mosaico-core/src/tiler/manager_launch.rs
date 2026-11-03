//! Launch bindings: starting the program a key is bound to, or bringing
//! an open window of it into view.

use crate::Result;
use crate::config::LaunchBinding;
use crate::system::Launcher;

use crate::tiler::raise::next_match;
use crate::tiler::{Input, Platform};

use super::TilingManager;

impl<P: Platform> TilingManager<P> {
    /// Carries out a press of `binding`.
    ///
    /// With `focus` set, a window the tiler manages that belongs to the
    /// application is focused and brought into view, the next one after
    /// the focused window when that is one already, so pressing again
    /// cycles through them. Otherwise, and when no window matches, the
    /// program is started with its arguments through `launcher`, and its
    /// window is taken in like any other once it is shown. Without
    /// `focus`, no window is looked at: every press starts the program.
    ///
    /// # Errors
    ///
    /// When `launcher` cannot start the program.
    pub fn launch(&mut self, binding: &LaunchBinding, launcher: &dyn Launcher) -> Result<()> {
        if binding.focus && self.raise_matching(binding) {
            return Ok(());
        }
        launcher.launch(&binding.command, &binding.args)
    }

    /// Raises a managed window that `binding` matches, and says whether
    /// there was one.
    fn raise_matching(&mut self, binding: &LaunchBinding) -> bool {
        let desktop = self.platform.desktop();
        let matching: Vec<_> = self
            .tiler
            .managed_windows()
            .into_iter()
            .filter(|&window| {
                desktop
                    .facts(window)
                    .is_some_and(|facts| binding.matches(&facts.class, &facts.title, &facts.path))
            })
            .collect();
        let Some(window) = next_match(&matching, self.tiler.focused_window()) else {
            return false;
        };
        self.step(Input::Raise(window));
        true
    }
}
