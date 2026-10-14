//! The process that owns a window.

use super::Window;

impl Window {
    /// The process that owns the window, or `None` when the window no
    /// longer exists.
    pub fn process_id(&self) -> Option<u32> {
        use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        let mut pid = 0u32;
        // SAFETY: GetWindowThreadProcessId only reads the owner of a
        // window handle into a caller-owned u32; it returns 0 for a handle
        // that is no longer valid.
        unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) };
        (pid != 0).then_some(pid)
    }

    /// Returns whether the process owning this window is running elevated (Administrator).
    ///
    /// Used to skip windows that mosaico cannot reposition due to UIPI
    /// when running as a regular user. When `OpenProcessToken` fails
    /// (access denied across the UIPI boundary), assumes elevated —
    /// if we can't query the token we can't control the window either.
    pub fn is_elevated(&self) -> bool {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::Security::{
            GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
        };
        use windows::Win32::System::Threading::{
            OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

        // SAFETY: Queries the process token elevation via standard Win32 APIs.
        // All handles (process, token) are closed before returning.
        unsafe {
            let mut pid = 0u32;
            GetWindowThreadProcessId(self.hwnd, Some(&mut pid));
            if pid == 0 {
                return false;
            }

            let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                // Cannot open the process — assume elevated so we don't
                // add an uncontrollable window to the tiling layout.
                return true;
            };

            let mut token = windows::Win32::Foundation::HANDLE::default();
            let token_ok = OpenProcessToken(process, TOKEN_QUERY, &mut token);
            let _ = CloseHandle(process);
            if token_ok.is_err() {
                // Can open the process but not its token → elevation
                // mismatch. We can't control this window either.
                return true;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut returned = 0u32;
            let info_ok = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned,
            );
            let _ = CloseHandle(token);

            info_ok.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    /// Returns the full path of the executable that owns this window,
    /// or an empty string if it cannot be queried.
    pub fn exe_path(&self) -> String {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        };
        use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        use windows::core::PWSTR;

        // SAFETY: Reads the image name into a caller-owned buffer. The
        // process handle is closed before returning.
        unsafe {
            let mut pid = 0u32;
            GetWindowThreadProcessId(self.hwnd, Some(&mut pid));
            if pid == 0 {
                return String::new();
            }
            let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                return String::new();
            };

            let mut buffer = [0u16; 1024];
            let mut len = buffer.len() as u32;
            let result = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(process);

            if result.is_err() {
                return String::new();
            }
            String::from_utf16_lossy(&buffer[..len as usize])
        }
    }
}
