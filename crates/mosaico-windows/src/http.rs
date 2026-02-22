//! Minimal HTTPS GET client using WinHTTP.
//!
//! Provides a single `get(host, path, timeout_ms)` function that makes a
//! synchronous HTTPS request and returns the response body as a string.

use std::ffi::c_void;

use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WinHttpCloseHandle, WinHttpConnect,
    WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable, WinHttpReadData,
    WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetTimeouts,
};
use windows::core::PCWSTR;

/// RAII wrapper for WinHTTP handles. Calls `WinHttpCloseHandle` on drop.
struct Handle(*mut c_void);

impl Handle {
    fn new(h: *mut c_void) -> Result<Self, String> {
        if h.is_null() {
            Err("WinHTTP returned null handle".into())
        } else {
            Ok(Self(h))
        }
    }

    fn ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = WinHttpCloseHandle(self.0);
            }
        }
    }
}

/// Performs a synchronous HTTPS GET and returns the response body.
///
/// `timeout_ms` applies independently to each WinHTTP phase (resolve,
/// connect, send, receive).  Returns `Err` on any network or protocol
/// failure; callers should treat errors as non-fatal.
pub fn get(host: &str, path: &str, timeout_ms: i32) -> Result<String, String> {
    let agent = to_wide(concat!("mosaico/", env!("CARGO_PKG_VERSION")));
    let host_w = to_wide(host);
    let path_w = to_wide(path);

    unsafe {
        let session = Handle::new(WinHttpOpen(
            PCWSTR(agent.as_ptr()),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            None,
            None,
            0,
        ))?;

        WinHttpSetTimeouts(
            session.ptr(),
            timeout_ms,
            timeout_ms,
            timeout_ms,
            timeout_ms,
        )
        .map_err(|e| e.message().to_string())?;

        let connect = Handle::new(WinHttpConnect(
            session.ptr(),
            PCWSTR(host_w.as_ptr()),
            443,
            0,
        ))?;

        let request = Handle::new(WinHttpOpenRequest(
            connect.ptr(),
            PCWSTR(to_wide("GET").as_ptr()),
            PCWSTR(path_w.as_ptr()),
            None,
            None,
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        ))?;

        WinHttpSendRequest(request.ptr(), None, None, 0, 0, 0)
            .map_err(|e| e.message().to_string())?;

        WinHttpReceiveResponse(request.ptr(), std::ptr::null_mut())
            .map_err(|e| e.message().to_string())?;

        read_body(request.ptr())
    }
}

/// Reads the full response body into a UTF-8 string.
unsafe fn read_body(request: *mut c_void) -> Result<String, String> {
    let mut body = Vec::new();
    loop {
        let mut available: u32 = 0;
        unsafe {
            WinHttpQueryDataAvailable(request, &mut available)
                .map_err(|e| e.message().to_string())?;
        }
        if available == 0 {
            break;
        }
        let mut buf = vec![0u8; available as usize];
        let mut read: u32 = 0;
        unsafe {
            WinHttpReadData(request, buf.as_mut_ptr().cast(), available, &mut read)
                .map_err(|e| e.message().to_string())?;
        }
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buf[..read as usize]);
    }
    String::from_utf8(body).map_err(|e| e.to_string())
}

/// Converts a `&str` to a null-terminated wide (UTF-16) string.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
