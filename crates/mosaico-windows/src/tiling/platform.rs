//! The Win32 desktop, as the tiling manager is driven against it.

use std::time::Instant;

use mosaico_core::tiler::{Desktop, Enumerator, Platform};
use mosaico_core::{Decorations, WindowControl};

use crate::decorations::Win32Decorations;
use crate::enumerate::WindowsEnumerator;
use crate::window_control::Win32WindowControl;

use super::desktop::Win32Desktop;

/// [`Platform`] on the Win32 desktop.
///
/// Reads windows and the cursor through Win32 queries, changes windows
/// through [`Win32WindowControl`], draws borders and corners with its
/// own layered overlays and DWM corner preferences, lists windows and
/// monitors through [`WindowsEnumerator`], and takes the time from the
/// system's monotonic clock.
pub struct Win32Platform {
    desktop: Win32Desktop,
    control: Win32WindowControl,
    decorations: Win32Decorations,
}

impl Win32Platform {
    /// The desktop as this process sees it. Whether mosaico runs
    /// elevated, which decides the windows it can reach, is read once
    /// here.
    pub fn new() -> Self {
        Self {
            desktop: Win32Desktop::new(),
            control: Win32WindowControl::default(),
            decorations: Win32Decorations::default(),
        }
    }

    /// Whether mosaico itself runs elevated.
    pub fn self_elevated(&self) -> bool {
        self.desktop.self_elevated()
    }
}

impl Default for Win32Platform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for Win32Platform {
    fn desktop(&self) -> &dyn Desktop {
        &self.desktop
    }

    fn control(&self) -> &dyn WindowControl {
        &self.control
    }

    fn decorations(&mut self) -> &mut dyn Decorations {
        &mut self.decorations
    }

    fn enumerator(&self) -> &dyn Enumerator {
        &WindowsEnumerator
    }

    fn now(&self) -> Instant {
        Instant::now()
    }
}
