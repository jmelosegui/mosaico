//! What the process's own access token says about it: the user who runs
//! mosaico, which names that user in the logon task and owns its pipes,
//! and its integrity level, which labels them.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, PSID, SID_AND_ATTRIBUTES, TOKEN_INFORMATION_CLASS, TOKEN_QUERY,
    TokenIntegrityLevel, TokenUser,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::PWSTR;

use mosaico_core::Result;

use crate::error::OrPlatform;

/// The current user's security identifier, as text like
/// `S-1-5-21-…-1001`.
pub fn current_user() -> Result<String> {
    own_token_sid(TokenUser, "find the current user")
}

/// The integrity level the process runs at, as the text of its label,
/// like `S-1-16-8192` for medium or `S-1-16-12288` for an elevated
/// process.
pub fn current_integrity() -> Result<String> {
    own_token_sid(TokenIntegrityLevel, "find the process's integrity level")
}

/// The SID that `class` reads from the process's own token.
fn own_token_sid(class: TOKEN_INFORMATION_CLASS, operation: &'static str) -> Result<String> {
    let mut token = HANDLE::default();
    // SAFETY: Opens the current process's own token for reading. It is
    // closed below.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .or_platform(operation)?;
    let sid = token_sid(token, class, operation);
    // SAFETY: Closes the token opened above, once.
    let _ = unsafe { CloseHandle(token) };
    sid
}

/// The SID that `class` reads from `token`, an open token with query
/// access. `class` must be one whose information starts with a
/// `SID_AND_ATTRIBUTES`, as `TokenUser` and `TokenIntegrityLevel` do.
fn token_sid(
    token: HANDLE,
    class: TOKEN_INFORMATION_CLASS,
    operation: &'static str,
) -> Result<String> {
    let mut size = 0u32;
    // SAFETY: Without a buffer, GetTokenInformation only writes the size
    // it needs to `size`, and fails because there is no buffer.
    let _ = unsafe { GetTokenInformation(token, class, None, 0, &mut size) };
    // A SID_AND_ATTRIBUTES followed by the SID it points to, in u64s so
    // the structure is aligned.
    let mut buffer = vec![0u64; (size as usize).div_ceil(size_of::<u64>())];
    // SAFETY: `buffer` holds at least `size` writable bytes.
    unsafe {
        GetTokenInformation(
            token,
            class,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        )
    }
    .or_platform(operation)?;
    // SAFETY: GetTokenInformation succeeded, so `buffer` starts with a
    // SID_AND_ATTRIBUTES (TOKEN_USER and TOKEN_MANDATORY_LABEL are one
    // each) whose SID points into `buffer`, which outlives it.
    let sid = unsafe { (*buffer.as_ptr().cast::<SID_AND_ATTRIBUTES>()).Sid };
    sid_text(sid).or_platform(operation)
}

/// `sid`, a valid SID, as text like `S-1-5-21-…-500`: always the SID
/// itself, never an alias such as `LA` that a security descriptor's text
/// uses for some accounts.
pub(crate) fn sid_text(sid: PSID) -> std::io::Result<String> {
    let mut text = PWSTR::null();
    // SAFETY: `sid` is a valid SID. The text it writes is freed below.
    unsafe { ConvertSidToStringSidW(sid, &mut text) }?;
    // SAFETY: `text` is the null-terminated string written above.
    let sid = unsafe { text.to_string() };
    // SAFETY: Frees the string ConvertSidToStringSidW allocated, once.
    unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
    sid.map_err(std::io::Error::other)
}
