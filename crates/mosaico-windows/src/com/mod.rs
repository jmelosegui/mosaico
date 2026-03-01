//! COM-based window cloaking via the ImmersiveShell.
//!
//! Uses the undocumented `IApplicationView::set_cloak` method to make
//! windows invisible without removing their taskbar icons or firing
//! Win32 hide/show events. This is the same mechanism Windows uses
//! for its built-in virtual desktop feature.

mod interfaces;

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::Interface;

use interfaces::{
    CLSID_ImmersiveShell, IApplicationViewCollectionVtbl, IApplicationViewVtbl,
    IID_SERVICE_PROVIDER, IID_VIEW_COLLECTION, IServiceProviderVtbl,
};

/// Ensures COM is initialized on the calling thread.
struct ComInit;

impl ComInit {
    fn new() -> Self {
        // SAFETY: CoInitializeEx is safe to call; duplicate calls on the
        // same thread return S_FALSE and are harmless.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }
        Self
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

thread_local! {
    static COM_INIT: ComInit = ComInit::new();
}

/// Cloaks a window, making it invisible while keeping its taskbar icon.
pub fn cloak_window(hwnd: HWND) {
    set_cloak(hwnd, 1, 2);
}

/// Uncloaks a previously cloaked window, restoring its visibility.
pub fn uncloak_window(hwnd: HWND) {
    set_cloak(hwnd, 1, 0);
}

/// Calls Release on a raw COM pointer.
///
/// # Safety
/// `ptr` must be a valid COM object pointer.
unsafe fn release(ptr: *mut c_void) {
    let vtbl = unsafe { &*(*(ptr as *const *const windows::core::IUnknown_Vtbl)) };
    unsafe { (vtbl.Release)(ptr) };
}

/// Calls `IApplicationView::set_cloak` via the ImmersiveShell COM API.
///
/// Errors are logged and silently ignored — cloaking is best-effort.
fn set_cloak(hwnd: HWND, cloak_type: u32, flags: i32) {
    COM_INIT.with(|_| {
        // Get IUnknown for the ImmersiveShell coclass.
        let unk: windows::core::IUnknown =
            match unsafe { CoCreateInstance(&CLSID_ImmersiveShell, None, CLSCTX_ALL) } {
                Ok(p) => p,
                Err(e) => {
                    mosaico_core::log_info!("COM ImmersiveShell failed: {e}");
                    return;
                }
            };

        // QueryInterface for IServiceProvider.
        let unk_ptr = unk.as_raw();
        let unk_vtbl = unsafe { &*(*(unk_ptr as *const *const windows::core::IUnknown_Vtbl)) };
        let mut sp_ptr: *mut c_void = std::ptr::null_mut();
        let hr = unsafe { (unk_vtbl.QueryInterface)(unk_ptr, &IID_SERVICE_PROVIDER, &mut sp_ptr) };
        if hr.is_err() || sp_ptr.is_null() {
            mosaico_core::log_info!("COM QI for IServiceProvider failed: {hr:?}");
            return;
        }

        // IServiceProvider::QueryService for IApplicationViewCollection.
        let sp_vtbl = unsafe { &*(*(sp_ptr as *const *const IServiceProviderVtbl)) };
        let mut col_ptr: *mut c_void = std::ptr::null_mut();
        let hr = unsafe {
            (sp_vtbl.query_service)(
                sp_ptr,
                &IID_VIEW_COLLECTION,
                &IID_VIEW_COLLECTION,
                &mut col_ptr,
            )
        };
        unsafe { release(sp_ptr) };
        if hr.is_err() || col_ptr.is_null() {
            mosaico_core::log_info!("COM QueryService for ViewCollection failed: {hr:?}");
            return;
        }

        // Get IApplicationView for the target window.
        let col_vtbl = unsafe { &*(*(col_ptr as *const *const IApplicationViewCollectionVtbl)) };
        let mut view_ptr: *mut c_void = std::ptr::null_mut();
        let hr = unsafe { (col_vtbl.get_view_for_hwnd)(col_ptr, hwnd, &mut view_ptr) };
        unsafe { release(col_ptr) };
        if hr.is_err() || view_ptr.is_null() {
            return;
        }

        // Call set_cloak on the application view.
        let view_vtbl = unsafe { &*(*(view_ptr as *const *const IApplicationViewVtbl)) };
        let _ = unsafe { (view_vtbl.set_cloak)(view_ptr, cloak_type, flags) };
        unsafe { release(view_ptr) };
    });
}
