//! COM-based window cloaking via the ImmersiveShell.
//!
//! Uses the undocumented `IApplicationView::set_cloak` method to make
//! windows invisible without removing their taskbar icons or firing
//! Win32 hide/show events. This is the same mechanism Windows uses
//! for its built-in virtual desktop feature.

mod interfaces;

use std::ffi::c_void;

use mosaico_core::Result;
use windows::Win32::Foundation::{E_POINTER, HWND};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::{HRESULT, Interface};

use crate::error::OrPlatform;

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
        // SAFETY: Paired with CoInitializeEx in `new`; one call per thread.
        unsafe {
            CoUninitialize();
        }
    }
}

thread_local! {
    static COM_INIT: ComInit = ComInit::new();
}

/// Cloaks a window, making it invisible while keeping its taskbar icon.
pub fn cloak_window(hwnd: HWND) -> Result<()> {
    set_cloak(hwnd, 1, 2).or_platform("cloak the window")
}

/// Uncloaks a previously cloaked window, restoring its visibility.
pub fn uncloak_window(hwnd: HWND) -> Result<()> {
    set_cloak(hwnd, 1, 0).or_platform("uncloak the window")
}

/// Calls Release on a raw COM pointer.
///
/// # Safety
/// `ptr` must be a valid COM object pointer.
unsafe fn release(ptr: *mut c_void) {
    // SAFETY: Caller guarantees `ptr` is a valid COM object with a standard vtbl layout.
    let vtbl = unsafe { &*(*(ptr as *const *const windows::core::IUnknown_Vtbl)) };
    // SAFETY: vtbl was validated above; Release decrements the refcount.
    unsafe { (vtbl.Release)(ptr) };
}

/// Calls `IApplicationView::set_cloak` via the ImmersiveShell COM API.
///
/// Fails with the error of the first step that does not work, such as
/// a window the shell has no view for. When the shell itself cannot be
/// reached, which affects every window, that is logged as well.
fn set_cloak(hwnd: HWND, cloak_type: u32, flags: i32) -> windows::core::Result<()> {
    COM_INIT.with(|_| {
        // SAFETY: COM is initialized on this thread via COM_INIT.
        let unk: windows::core::IUnknown =
            match unsafe { CoCreateInstance(&CLSID_ImmersiveShell, None, CLSCTX_ALL) } {
                Ok(p) => p,
                Err(e) => {
                    mosaico_core::log_info!("COM ImmersiveShell failed: {e}");
                    return Err(e);
                }
            };

        // SAFETY: `unk` is a valid COM object from CoCreateInstance; vtbl access is sound.
        let unk_ptr = unk.as_raw();
        // SAFETY: unk_ptr is a valid COM object from CoCreateInstance.
        let unk_vtbl = unsafe { &*(*(unk_ptr as *const *const windows::core::IUnknown_Vtbl)) };
        let mut sp_ptr: *mut c_void = std::ptr::null_mut();
        // SAFETY: QueryInterface on a valid IUnknown vtbl.
        let hr = unsafe { (unk_vtbl.QueryInterface)(unk_ptr, &IID_SERVICE_PROVIDER, &mut sp_ptr) };
        if hr.is_err() || sp_ptr.is_null() {
            mosaico_core::log_info!("COM QI for IServiceProvider failed: {hr:?}");
            return Err(failed(hr));
        }

        // SAFETY: `sp_ptr` is a valid IServiceProvider from the QI above; released after use.
        let sp_vtbl = unsafe { &*(*(sp_ptr as *const *const IServiceProviderVtbl)) };
        let mut col_ptr: *mut c_void = std::ptr::null_mut();
        // SAFETY: query_service on a valid IServiceProvider vtbl.
        let hr = unsafe {
            (sp_vtbl.query_service)(
                sp_ptr,
                &IID_VIEW_COLLECTION,
                &IID_VIEW_COLLECTION,
                &mut col_ptr,
            )
        };
        // SAFETY: sp_ptr is no longer needed; release its refcount.
        unsafe { release(sp_ptr) };
        if hr.is_err() || col_ptr.is_null() {
            mosaico_core::log_info!("COM QueryService for ViewCollection failed: {hr:?}");
            return Err(failed(hr));
        }

        // SAFETY: `col_ptr` is a valid IApplicationViewCollection; released after use.
        let col_vtbl = unsafe { &*(*(col_ptr as *const *const IApplicationViewCollectionVtbl)) };
        let mut view_ptr: *mut c_void = std::ptr::null_mut();
        // SAFETY: get_view_for_hwnd on a valid IApplicationViewCollection.
        let hr = unsafe { (col_vtbl.get_view_for_hwnd)(col_ptr, hwnd, &mut view_ptr) };
        // SAFETY: col_ptr is no longer needed; release its refcount.
        unsafe { release(col_ptr) };
        if hr.is_err() || view_ptr.is_null() {
            return Err(failed(hr));
        }

        // SAFETY: `view_ptr` is a valid IApplicationView; released after use.
        let view_vtbl = unsafe { &*(*(view_ptr as *const *const IApplicationViewVtbl)) };
        // SAFETY: set_cloak on a valid IApplicationView vtbl.
        let hr = unsafe { (view_vtbl.set_cloak)(view_ptr, cloak_type, flags) };
        // SAFETY: view_ptr is no longer needed; release its refcount.
        unsafe { release(view_ptr) };
        hr.ok()
    })
}

/// The error of a COM call that failed, or that reported success but
/// handed back no object.
fn failed(hr: HRESULT) -> windows::core::Error {
    if hr.is_err() {
        hr.into()
    } else {
        E_POINTER.into()
    }
}
