//! The record of the windows workspace switches hid, kept up to
//! date and on disk.

use std::time::Instant;

use mosaico_core::tiler::Desktop;
use mosaico_core::{WindowControl, WindowId};

use super::Applier;

impl Applier {
    /// Shows again the windows a previous daemon hid and never showed,
    /// because it was killed or crashed, then clears its record.
    ///
    /// Run before the first windows are enumerated, so they are adopted
    /// like any other window.
    pub(crate) fn restore_previous(
        &mut self,
        desktop: &dyn Desktop,
        control: &dyn WindowControl,
        now: Instant,
    ) {
        let Some(file) = &self.hidden_file else {
            return;
        };
        let effects = file.load().restore(|w| desktop.identity(w));
        if !effects.is_empty() {
            mosaico_core::log_info!(
                "showing {} window(s) a previous daemon left hidden",
                effects.len()
            );
        }
        for effect in &effects {
            self.run(effect, desktop, control, now);
        }
        self.save_hidden();
    }

    /// Forgets a window that was destroyed.
    pub(crate) fn forget(&mut self, window: WindowId) {
        if self.hidden.forget(window) {
            self.save_hidden();
        }
    }
}
