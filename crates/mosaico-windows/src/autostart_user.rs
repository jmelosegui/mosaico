//! The security identifier of the user who runs mosaico, which names
//! that user in the logon task.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::PWSTR;

use mosaico_core::Result;

use crate::error::OrPlatform;

const OPERATION: &str = "find the current user";

/// The current user's security identifier, as text like
/// `S-1-5-21-…-1001`.
pub fn current_user() -> Result<String> {
    let mut token = HANDLE::default();
    // SAFETY: Opens the current process's own token for reading. It is
    // closed below.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .or_platform(OPERATION)?;
    let user = token_user(token);
    // SAFETY: Closes the token opened above, once.
    let _ = unsafe { CloseHandle(token) };
    user
}

/// The user of `token`, an open token with query access.
fn token_user(token: HANDLE) -> Result<String> {
    let mut size = 0u32;
    // SAFETY: Without a buffer, GetTokenInformation only writes the size
    // it needs to `size`, and fails because there is no buffer.
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut size) };
    // A TOKEN_USER followed by the SID it points to, in u64s so the
    // TOKEN_USER is aligned.
    let mut buffer = vec![0u64; (size as usize).div_ceil(size_of::<u64>())];
    // SAFETY: `buffer` holds at least `size` writable bytes.
    unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        )
    }
    .or_platform(OPERATION)?;
    // SAFETY: GetTokenInformation succeeded, so `buffer` starts with a
    // TOKEN_USER whose SID points into `buffer`, which outlives it.
    let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut text = PWSTR::null();
    // SAFETY: `sid` is a valid SID. The text it writes is freed below.
    unsafe { ConvertSidToStringSidW(sid, &mut text) }.or_platform(OPERATION)?;
    // SAFETY: `text` is the null-terminated string written above.
    let user = unsafe { text.to_string() };
    // SAFETY: Frees the string ConvertSidToStringSidW allocated, once.
    unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
    user.or_platform(OPERATION)
}
