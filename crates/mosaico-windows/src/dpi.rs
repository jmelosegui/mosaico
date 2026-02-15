use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};

/// Declares this process as per-monitor DPI aware (V2).
///
/// Without this, Windows scales coordinates for us based on the primary
/// monitor's DPI, which gives wrong positions on mixed-DPI setups. With
/// per-monitor awareness, we get raw pixel coordinates and can position
/// windows accurately on any monitor.
///
/// Must be called once at process startup, before creating any windows
/// or calling any Win32 functions that depend on DPI.
pub fn enable_dpi_awareness() {
    // SAFETY: SetProcessDpiAwarenessContext is safe to call once at startup.
    // If it fails (e.g. already set via manifest), we ignore the error.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}
