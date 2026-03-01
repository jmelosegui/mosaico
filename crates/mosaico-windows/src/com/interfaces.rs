// COM interface definitions for the Windows ImmersiveShell.
//
// These undocumented interfaces provide access to the same cloaking
// mechanism that Windows uses for virtual desktops. The vtable
// layouts are defined manually to avoid an external proc-macro
// dependency.
//
// Interface definitions are based on the AltTabAccessor project by
// Jari Pennanen (Ciantic), available under the MIT license at:
// https://github.com/Ciantic/AltTabAccessor

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::core::{GUID, HRESULT, IUnknown_Vtbl};

#[allow(non_upper_case_globals)]
pub const CLSID_ImmersiveShell: GUID = GUID {
    data1: 0xC2F0_3A33,
    data2: 0x21F5,
    data3: 0x47FA,
    data4: [0xB4, 0xBB, 0x15, 0x63, 0x62, 0xA2, 0xF2, 0x39],
};

pub const IID_SERVICE_PROVIDER: GUID = GUID {
    data1: 0x6D51_40C1,
    data2: 0x7436,
    data3: 0x11CE,
    data4: [0x80, 0x34, 0x00, 0xAA, 0x00, 0x60, 0x09, 0xFA],
};

pub const IID_VIEW_COLLECTION: GUID = GUID {
    data1: 0x1841_C6D7,
    data2: 0x4F9D,
    data3: 0x42C0,
    data4: [0xAF, 0x41, 0x87, 0x47, 0x53, 0x8F, 0x10, 0xE5],
};

// IServiceProvider {6D5140C1-7436-11CE-8034-00AA006009FA}
#[repr(C)]
pub struct IServiceProviderVtbl {
    pub base: IUnknown_Vtbl,
    pub query_service: unsafe extern "system" fn(
        this: *mut c_void,
        guid_service: *const GUID,
        riid: *const GUID,
        ppv_object: *mut *mut c_void,
    ) -> HRESULT,
}

// IApplicationViewCollection {1841C6D7-4F9D-42C0-AF41-8747538F10E5}
#[repr(C)]
pub struct IApplicationViewCollectionVtbl {
    pub base: IUnknown_Vtbl,
    pub get_views: unsafe extern "system" fn(this: *mut c_void, out: *mut c_void) -> HRESULT,
    pub get_views_by_zorder:
        unsafe extern "system" fn(this: *mut c_void, out: *mut c_void) -> HRESULT,
    pub get_views_by_app_user_model_id:
        unsafe extern "system" fn(this: *mut c_void, id: *mut c_void, out: *mut c_void) -> HRESULT,
    pub get_view_for_hwnd: unsafe extern "system" fn(
        this: *mut c_void,
        window: HWND,
        out_view: *mut *mut c_void,
    ) -> HRESULT,
}

// IApplicationView {372E1D3B-38D3-42E4-A15B-8AB2B178F513}
//
// Only methods up to `set_cloak` are declared. Later vtable entries
// are not needed and omitting them is safe for COM call dispatch.
#[repr(C)]
pub struct IApplicationViewVtbl {
    pub base: IUnknown_Vtbl,
    // IInspectable (3 methods)
    pub get_iids: unsafe extern "system" fn(
        this: *mut c_void,
        out_count: *mut u32,
        out: *mut *mut GUID,
    ) -> HRESULT,
    pub get_runtime_class_name:
        unsafe extern "system" fn(this: *mut c_void, out: *mut c_void) -> HRESULT,
    pub get_trust_level: unsafe extern "system" fn(this: *mut c_void, out: *mut c_void) -> HRESULT,
    // IApplicationView (7 methods up to set_cloak)
    pub set_focus: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub switch_to: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub try_invoke_back: unsafe extern "system" fn(this: *mut c_void, cb: u32) -> HRESULT,
    pub get_thumbnail_window:
        unsafe extern "system" fn(this: *mut c_void, out: *mut HWND) -> HRESULT,
    pub get_monitor: unsafe extern "system" fn(this: *mut c_void, out: *mut *mut u32) -> HRESULT,
    pub get_visibility: unsafe extern "system" fn(this: *mut c_void, out: *mut c_void) -> HRESULT,
    pub set_cloak:
        unsafe extern "system" fn(this: *mut c_void, cloak_type: u32, flags: i32) -> HRESULT,
}
