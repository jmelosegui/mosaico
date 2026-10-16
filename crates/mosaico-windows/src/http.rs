//! [`HttpClient`](mosaico_core::http::HttpClient) through WinHTTP.
//!
//! Each request opens its own synchronous WinHTTP session. WinHTTP
//! follows redirects by itself (by default from HTTPS only to HTTPS),
//! which GitHub release downloads rely on: they redirect to
//! `objects.githubusercontent.com`.

#[path = "http_response.rs"]
mod response;

use std::ffi::c_void;
use std::time::Duration;

use mosaico_core::http::{self, HttpClient};
use mosaico_core::{Error, Result};
use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WinHttpCloseHandle, WinHttpConnect,
    WinHttpOpen, WinHttpOpenRequest, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetTimeouts,
};
use windows::core::PCWSTR;

use crate::error::OrPlatform;

/// [`HttpClient`] through WinHTTP.
///
/// It holds no state, so it is `Send` and `Sync`. The daemon relies on
/// that: it moves the client into the background threads that look up
/// the latest release and download the community rules.
#[derive(Debug, Clone, Copy, Default)]
pub struct WinHttpClient;

impl HttpClient for WinHttpClient {
    /// Sends the request on port 443 and checks the status before
    /// reading the body. `timeout` becomes WinHTTP's resolve, connect,
    /// send and receive timeouts, each of which bounds one step.
    ///
    /// Opening the session, the connection and the request handles, and
    /// reading the status, involve no network, so their failures are
    /// platform errors. Sending the request and receiving the answer are
    /// the network steps.
    fn get_bytes(&self, host: &str, path: &str, timeout: Duration) -> Result<Vec<u8>> {
        let url = http::url(host, path);
        let agent = to_wide(concat!("mosaico/", env!("CARGO_PKG_VERSION")));
        let host_w = to_wide(host);
        let path_w = to_wide(path);
        let verb = to_wide("GET");
        let timeout = timeout_ms(timeout);

        // SAFETY: the session, connection and request handles are opened
        // in sequence and wrapped in `Handle`, which closes them on drop,
        // request first. Every wide string outlives the calls that read
        // it.
        unsafe {
            let session = Handle::new(WinHttpOpen(
                PCWSTR(agent.as_ptr()),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                None,
                None,
                0,
            ))
            .or_platform("open an HTTP session")?;

            WinHttpSetTimeouts(session.ptr(), timeout, timeout, timeout, timeout)
                .or_platform("set the HTTP timeouts")?;

            let connect = Handle::new(WinHttpConnect(
                session.ptr(),
                PCWSTR(host_w.as_ptr()),
                443,
                0,
            ))
            .or_platform("set up the HTTP connection")?;

            let request = Handle::new(WinHttpOpenRequest(
                connect.ptr(),
                PCWSTR(verb.as_ptr()),
                PCWSTR(path_w.as_ptr()),
                None,
                None,
                std::ptr::null(),
                WINHTTP_FLAG_SECURE,
            ))
            .or_platform("set up the HTTP request")?;

            WinHttpSendRequest(request.ptr(), None, None, 0, 0, 0)
                .map_err(failed(&url, "send the request"))?;
            WinHttpReceiveResponse(request.ptr(), std::ptr::null_mut())
                .map_err(failed(&url, "receive the answer"))?;

            let status = response::status(request.ptr()).or_platform("read the HTTP status")?;
            check_status(&url, status)?;
            response::read_body(request.ptr()).map_err(failed(&url, "read the answer"))
        }
    }
}

/// RAII wrapper for WinHTTP handles. Calls `WinHttpCloseHandle` on drop.
struct Handle(*mut c_void);

impl Handle {
    /// Takes `handle`, or the error of the call that returned it null.
    fn new(handle: *mut c_void) -> windows::core::Result<Self> {
        if handle.is_null() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(Self(handle))
        }
    }

    fn ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: WinHttpCloseHandle closes a WinHTTP handle. The pointer
        // is non-null (checked in `new`) and exclusively owned.
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

/// Turns the error of a failed WinHTTP call during `step` into a failed
/// request for `url`.
fn failed<'a>(url: &'a str, step: &'static str) -> impl FnOnce(windows::core::Error) -> Error + 'a {
    move |e| Error::http_request(url, step, e)
}

/// Fails a request whose answer has a status outside 200-299, so an
/// error page is never taken for the file that was asked for.
fn check_status(url: &str, status: u32) -> Result<()> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(Error::http_status(
            url,
            u16::try_from(status).unwrap_or(u16::MAX),
        ))
    }
}

/// `timeout` in the whole milliseconds WinHTTP takes: at least 1, since
/// WinHTTP takes 0 as no timeout at all, and at most `i32::MAX`.
fn timeout_ms(timeout: Duration) -> i32 {
    i32::try_from(timeout.as_millis())
        .unwrap_or(i32::MAX)
        .max(1)
}

/// Converts a `&str` to a null-terminated wide (UTF-16) string.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
