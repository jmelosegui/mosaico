//! Who may open the daemon's pipes, and how a client makes sure the pipe
//! it opened is the daemon's.
//!
//! Without an explicit descriptor, a named pipe lets Everyone and
//! Anonymous read it. Every instance is created with one that names the
//! user and the system, denies network logons and carries a mandatory
//! label at the creating process's integrity level with no-read-up and
//! no-write-up, so less trusted processes of the user can neither read
//! nor write. The owner is the user rather than the token's default
//! owner, which is the Administrators group for an elevated process, so
//! a client can check it whether the daemon is elevated or not.
//!
//! A client opens the pipe at the identification level, so a server
//! that is not the daemon cannot act as the user, and refuses a pipe
//! owned by another account or labelled below medium.

use std::io;
use std::os::windows::io::AsRawHandle;

use windows::Win32::Foundation::{HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW, GetSecurityInfo, SDDL_REVISION_1,
    SE_KERNEL_OBJECT,
};
use windows::Win32::Security::{
    GetSecurityDescriptorOwner, LABEL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES,
};
use windows::core::{BOOL, HSTRING, PWSTR};

use mosaico_core::ipc::IpcError;

use crate::token;

/// The integrity level below which a pipe is not trusted: medium, what
/// the user's ordinary programs run at.
const MEDIUM_INTEGRITY: u32 = 0x2000;

/// A security descriptor, for a pipe instance to be created or read from
/// an open one, freed when dropped.
pub(crate) struct PipeSecurity {
    descriptor: PSECURITY_DESCRIPTOR,
}

impl PipeSecurity {
    /// Lets in the current user and the system, never over the network,
    /// and nothing less trusted than this process.
    pub(crate) fn for_this_process() -> io::Result<Self> {
        let user = token::current_user().map_err(io::Error::other)?;
        let label = token::current_integrity().map_err(io::Error::other)?;
        Self::from_text(&descriptor_text(&user, &label))
    }

    /// Lets in the current user and the system, never over the network,
    /// and nothing less trusted than the user's ordinary programs, whatever
    /// this process's own level.
    pub(crate) fn at_medium() -> io::Result<Self> {
        let user = token::current_user().map_err(io::Error::other)?;
        Self::from_text(&descriptor_text(&user, "ME"))
    }

    /// The descriptor written as `text`, in the security descriptor
    /// definition language.
    pub(crate) fn from_text(text: &str) -> io::Result<Self> {
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        // SAFETY: Parses `text` into a descriptor the system allocates;
        // it is freed once, when this value is dropped.
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                &HSTRING::from(text),
                SDDL_REVISION_1,
                &mut descriptor,
                None,
            )
        }?;
        Ok(Self { descriptor })
    }

    /// The owner and label of the object behind `handle`.
    fn of(handle: HANDLE) -> io::Result<Self> {
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        // SAFETY: Reads the owner and label of an open handle into a
        // descriptor the system allocates; it is freed once, when this
        // value is dropped.
        unsafe {
            GetSecurityInfo(
                handle,
                SE_KERNEL_OBJECT,
                OWNER_SECURITY_INFORMATION | LABEL_SECURITY_INFORMATION,
                None,
                None,
                None,
                None,
                Some(&mut descriptor),
            )
        }
        .ok()?;
        Ok(Self { descriptor })
    }

    /// Whether this is the descriptor of a pipe the daemon of `user`
    /// created: owned by `user` and labelled medium or above.
    fn trusted_by(&self, user: &str) -> io::Result<bool> {
        Ok(trusts(self.owner()?.as_deref(), &self.text()?, user))
    }

    /// The owner's SID, or `None` without one. Read from the descriptor
    /// itself, because its text names some accounts, such as the built-in
    /// Administrator, by an alias instead of their SID.
    fn owner(&self) -> io::Result<Option<String>> {
        let mut owner = PSID::default();
        let mut defaulted = BOOL(0);
        // SAFETY: Reads the owner of a valid descriptor; `owner` points
        // into the descriptor, which outlives its use below.
        unsafe { GetSecurityDescriptorOwner(self.descriptor, &mut owner, &mut defaulted) }?;
        if owner.is_invalid() {
            return Ok(None);
        }
        token::sid_text(owner).map(Some)
    }

    /// The owner and label, in the security descriptor definition
    /// language.
    fn text(&self) -> io::Result<String> {
        let mut text = PWSTR::null();
        // SAFETY: `self.descriptor` is a valid descriptor; the text it
        // writes is freed below.
        unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                self.descriptor,
                SDDL_REVISION_1,
                OWNER_SECURITY_INFORMATION | LABEL_SECURITY_INFORMATION,
                &mut text,
                None,
            )
        }?;
        // SAFETY: `text` is the null-terminated string written above.
        let owned = unsafe { text.to_string() };
        // SAFETY: Frees the string written above, once.
        unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
        owned.map_err(io::Error::other)
    }

    /// Attributes that apply this descriptor and are not inherited. They
    /// point into `self`, which must outlive their use.
    pub(crate) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.descriptor.0,
            bInheritHandle: BOOL(0),
        }
    }
}

impl Drop for PipeSecurity {
    fn drop(&mut self) {
        // SAFETY: Frees the descriptor allocated in `from_text` or `of`,
        // once.
        unsafe { LocalFree(Some(HLOCAL(self.descriptor.0))) };
    }
}

/// The descriptor of a pipe that `user` owns and only `user` and the
/// system may use, never from another machine, labelled `label` so
/// nothing below it may read or write.
fn descriptor_text(user: &str, label: &str) -> String {
    format!("O:{user}D:P(D;;GA;;;NU)(A;;GA;;;SY)(A;;GA;;;{user})S:(ML;;NWNR;;;{label})")
}

/// Fails with [`IpcError::Untrusted`] unless the pipe `file` is connected
/// to belongs to the current user and is labelled medium or above.
pub(crate) fn check_server(file: &impl AsRawHandle) -> Result<(), IpcError> {
    let security = PipeSecurity::of(HANDLE(file.as_raw_handle()))?;
    let user = token::current_user().map_err(io::Error::other)?;
    if security.trusted_by(&user)? {
        Ok(())
    } else {
        let text = security.text().unwrap_or_default();
        mosaico_core::log_warn!("IPC: refused a pipe that is not the daemon's: {text}");
        Err(IpcError::Untrusted)
    }
}

/// Whether a pipe owned by `owner`, whose label reads `text`, is one the
/// daemon of `user` created: owned by `user` and labelled medium or
/// above. A pipe with no label counts as medium, as the system treats it.
fn trusts(owner: Option<&str>, text: &str, user: &str) -> bool {
    owner == Some(user) && label(text).is_none_or(|level| level >= MEDIUM_INTEGRITY)
}

/// The integrity level of the mandatory label in a descriptor's text, or
/// `None` without one. A label that cannot be read counts as untrusted.
fn label(text: &str) -> Option<u32> {
    let ace = &text[text.find("(ML;")?..];
    let ace = &ace[..ace.find(')').unwrap_or(ace.len())];
    let sid = ace.rsplit(';').next().unwrap_or_default();
    Some(integrity(sid).unwrap_or(0))
}

/// The integrity level an integrity label SID or its alias names.
fn integrity(sid: &str) -> Option<u32> {
    match sid {
        "LW" => Some(0x1000),
        "ME" => Some(0x2000),
        "MP" => Some(0x2100),
        "HI" => Some(0x3000),
        "SI" => Some(0x4000),
        _ => sid.strip_prefix("S-1-16-")?.parse().ok(),
    }
}

#[cfg(test)]
#[path = "ipc_security_tests.rs"]
mod tests;
