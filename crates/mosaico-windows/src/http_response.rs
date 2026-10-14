//! Reading the answer to a WinHTTP request.

use std::ffi::c_void;

use windows::Win32::Networking::WinHttp::{
    WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE, WinHttpQueryDataAvailable,
    WinHttpQueryHeaders, WinHttpReadData,
};
use windows::core::PCWSTR;

/// The HTTP status code of the answer to `request`.
///
/// # Safety
///
/// `request` must be an open request handle whose answer was received.
pub(super) unsafe fn status(request: *mut c_void) -> windows::core::Result<u32> {
    let mut status: u32 = 0;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: the caller passes an open request handle. The buffer is a
    // `u32` of the size passed, which is what `WINHTTP_QUERY_FLAG_NUMBER`
    // writes, and no header name or index is asked for.
    unsafe {
        WinHttpQueryHeaders(
            request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some((&raw mut status).cast()),
            &mut size,
            std::ptr::null_mut(),
        )?;
    }
    Ok(status)
}

/// Reads the full body of the answer to `request`.
///
/// # Safety
///
/// `request` must be an open request handle whose answer was received.
pub(super) unsafe fn read_body(request: *mut c_void) -> windows::core::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let mut available: u32 = 0;
        // SAFETY: the caller passes an open request handle.
        unsafe {
            WinHttpQueryDataAvailable(request, &mut available)?;
        }
        if available == 0 {
            break;
        }
        let mut buf = vec![0u8; available as usize];
        let mut read: u32 = 0;
        // SAFETY: WinHttpReadData reads at most `available` bytes into a
        // buffer of that size.
        unsafe {
            WinHttpReadData(request, buf.as_mut_ptr().cast(), available, &mut read)?;
        }
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buf[..read as usize]);
    }
    Ok(body)
}
