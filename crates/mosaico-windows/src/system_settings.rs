//! The system settings that change what mosaico does.

use std::ffi::c_void;

use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_REMOTESESSION, SPI_GETCLIENTAREAANIMATION,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::BOOL;

/// Whether windows may be animated: "Animation effects" is on in the
/// Windows settings, "Show animations in Windows" on Windows 10, and the
/// session is not a remote one.
pub(crate) fn animations_allowed() -> bool {
    allowed(client_area_animation(), remote_session())
}

/// Whether windows may be animated, from the animation setting, `None`
/// when it cannot be read, and whether the session is remote. A setting
/// that cannot be read allows them, as on a system without one.
fn allowed(setting: Option<bool>, remote: bool) -> bool {
    !remote && setting.unwrap_or(true)
}

/// `SPI_GETCLIENTAREAANIMATION`, the setting behind both toggles.
fn client_area_animation() -> Option<bool> {
    let mut on = BOOL::default();
    // SAFETY: SystemParametersInfoW writes a BOOL into `on`, which
    // outlives the call, and changes no setting.
    unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&raw mut on).cast::<c_void>()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()?;
    Some(on.as_bool())
}

/// Whether this is a remote session, `SM_REMOTESESSION`, where every
/// frame would be sent over the network.
fn remote_session() -> bool {
    // SAFETY: GetSystemMetrics is a read-only query.
    unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_animate_as_the_setting_says() {
        // Arrange / Act / Assert
        assert!(allowed(Some(true), false));
        assert!(!allowed(Some(false), false));
    }

    #[test]
    fn a_setting_that_cannot_be_read_lets_windows_animate() {
        // Arrange / Act / Assert
        assert!(allowed(None, false));
    }

    #[test]
    fn windows_never_animate_in_a_remote_session() {
        // Arrange / Act / Assert
        assert!(!allowed(Some(true), true));
        assert!(!allowed(None, true));
    }

    #[test]
    fn the_animation_setting_can_be_read() {
        // Act / Assert -- it is only read.
        assert!(client_area_animation().is_some());
    }
}
