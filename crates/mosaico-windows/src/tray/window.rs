//! The hidden window that owns the icon and hears what the user does on
//! it.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, PoisonError};

use mosaico_core::tray::{TrayCommand, TrayNotice, TrayState};
use mosaico_core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NIM_SETVERSION, NOTIFY_ICON_MESSAGE, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    ChangeWindowMessageFilterEx, CreateWindowExW, DefWindowProcW, DestroyWindow, MSGFLT_ALLOW,
    PostMessageW, RegisterClassW, RegisterWindowMessageW, WM_APP, WNDCLASSW, WS_EX_TOOLWINDOW,
};
use windows::core::{PCWSTR, w};

use super::decode::{Click, click, copy_wide};
use super::icon::Icons;
use super::menu;
use crate::error::OrPlatform;

/// The name of the hidden window's class.
const CLASS_NAME: PCWSTR = w!("MosaicoTray");

/// The id of the one icon the window owns.
const ICON_ID: u32 = 1;

/// What the shell sends the window about the icon.
const WM_TRAY_ICON: u32 = WM_APP + 1;

/// What the watch sends the window when it queued a request.
const WM_TRAY_REQUEST: u32 = WM_APP + 2;

/// Whether this process registered the class, so that an icon shown
/// again reuses it.
static REGISTERED: Mutex<bool> = Mutex::new(false);

/// The message the shell broadcasts when it starts, after which every
/// icon has to be added again. Zero until the class is registered.
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// The icon of the tray thread, for its window procedure.
    static WINDOW: RefCell<Option<TrayWindow>> = const { RefCell::new(None) };
}

/// What the watch asks of the tray thread.
pub(super) enum Request {
    /// Show this state.
    Update(TrayState),
    /// Show this notice beside the icon.
    Notify(TrayNotice),
}

/// Wakes the tray thread to read its requests, from any thread.
pub(super) struct Waker(usize);

impl Waker {
    pub(super) fn wake(&self) -> Result<()> {
        let hwnd = HWND(self.0 as *mut _);
        // SAFETY: PostMessageW only queues a message for the window; a
        // window that is gone makes it fail, which is reported.
        unsafe { PostMessageW(Some(hwnd), WM_TRAY_REQUEST, WPARAM(0), LPARAM(0)) }
            .or_platform("wake the tray icon")
    }
}

/// The icon and the hidden window it reports to.
pub(super) struct TrayWindow {
    hwnd: HWND,
    state: TrayState,
    commands: Sender<TrayCommand>,
    inbox: Receiver<Request>,
    icons: Icons,
}

impl TrayWindow {
    /// Creates the window, shows the icon for `state` and keeps both for
    /// the calling thread's window procedure, until [`close`].
    pub(super) fn open(
        state: TrayState,
        commands: Sender<TrayCommand>,
        inbox: Receiver<Request>,
    ) -> Result<Waker> {
        register_class()?;
        // SAFETY: Creates a hidden window of the class registered above.
        // WS_EX_TOOLWINDOW: no taskbar entry. No WS_VISIBLE: it stays
        // hidden. Not message-only, so it hears the shell's broadcast.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                CLASS_NAME,
                w!("mosaico"),
                Default::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            )
        }
        .or_platform("create the tray window")?;
        let window = Self {
            hwnd,
            state,
            commands,
            inbox,
            icons: Icons::load(),
        };
        // SAFETY: Lets the shell's broadcast through when mosaico runs
        // elevated; the window is the one just created.
        unsafe {
            let taskbar_created = TASKBAR_CREATED.load(Ordering::Relaxed);
            let _ = ChangeWindowMessageFilterEx(hwnd, taskbar_created, MSGFLT_ALLOW, None);
        }
        window.add()?;
        WINDOW.with(|cell| *cell.borrow_mut() = Some(window));
        Ok(Waker(hwnd.0 as usize))
    }

    /// Adds the icon to the notification area.
    fn add(&self) -> Result<()> {
        let mut data = self.data();
        notify_icon(NIM_ADD, &data, "show the tray icon")?;
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        notify_icon(NIM_SETVERSION, &data, "set up the tray icon")
    }

    /// The icon, its tooltip and where it reports, for the current state.
    fn data(&self) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: ICON_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_TRAY_ICON,
            hIcon: self.icons.get(self.state.paused),
            ..Default::default()
        };
        copy_wide(&mut data.szTip, &self.state.tooltip());
        data
    }

    /// Carries out what the watch queued, in order.
    fn handle_requests(&mut self) {
        while let Ok(request) = self.inbox.try_recv() {
            let done = match request {
                Request::Update(state) => {
                    self.state = state;
                    notify_icon(NIM_MODIFY, &self.data(), "update the tray icon")
                }
                Request::Notify(notice) => {
                    let mut data = self.data();
                    data.uFlags |= NIF_INFO;
                    data.dwInfoFlags = NIIF_INFO;
                    copy_wide(&mut data.szInfoTitle, &notice.title);
                    copy_wide(&mut data.szInfo, &notice.body);
                    notify_icon(NIM_MODIFY, &data, "show a tray notification")
                }
            };
            if let Err(e) = done {
                mosaico_core::log_warn!("{e}");
            }
        }
    }

    fn send(&self, command: TrayCommand) {
        // A daemon that stopped listening is going away; nothing to do.
        let _ = self.commands.send(command);
    }
}

impl Drop for TrayWindow {
    fn drop(&mut self) {
        let _ = notify_icon(NIM_DELETE, &self.data(), "remove the tray icon");
        // SAFETY: Called on the tray thread, which created the window; a
        // TrayWindow never leaves it.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Removes the calling thread's icon and destroys its window.
pub(super) fn close() {
    let window = WINDOW.with(|cell| cell.borrow_mut().take());
    drop(window);
}

/// Runs `f` on the calling thread's icon, unless there is none or it is
/// in use further up the stack.
fn with_window<R>(f: impl FnOnce(&mut TrayWindow) -> R) -> Option<R> {
    WINDOW.with(|cell| {
        let mut window = cell.try_borrow_mut().ok()?;
        window.as_mut().map(f)
    })
}

fn notify_icon(
    message: NOTIFY_ICON_MESSAGE,
    data: &NOTIFYICONDATAW,
    operation: &'static str,
) -> Result<()> {
    // SAFETY: The data is a complete NOTIFYICONDATAW whose size field is
    // set and whose window belongs to this thread.
    if unsafe { Shell_NotifyIconW(message, data) }.as_bool() {
        Ok(())
    } else {
        Err(Error::platform(operation, "the shell refused the request"))
    }
}

/// Registers the hidden window's class, once per process.
fn register_class() -> Result<()> {
    let mut registered = REGISTERED.lock().unwrap_or_else(PoisonError::into_inner);
    if *registered {
        return Ok(());
    }
    // SAFETY: RegisterWindowMessageW takes a static string and returns
    // the same id to every caller in the session.
    let taskbar_created = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    TASKBAR_CREATED.store(taskbar_created, Ordering::Relaxed);
    let wc = WNDCLASSW {
        lpfnWndProc: Some(tray_proc),
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };
    // SAFETY: RegisterClassW copies the class description, whose name is
    // a static string and whose window procedure is a static function.
    if unsafe { RegisterClassW(&wc) } == 0 {
        return Err(Error::platform(
            "register the tray window class",
            windows::core::Error::from_win32(),
        ));
    }
    *registered = true;
    Ok(())
}

/// WNDPROC for the hidden window.
unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY_ICON => {
            match click(wparam.0, lparam.0) {
                Some(Click::Default) => {
                    with_window(|window| window.send(TrayCommand::DEFAULT));
                }
                Some(Click::Menu(x, y)) => {
                    // The menu runs a loop of its own, so the icon is not
                    // held while it is open.
                    let items = with_window(|window| window.state.menu());
                    let picked = items.and_then(|items| menu::pick(hwnd, &items, x, y));
                    if let Some(command) = picked {
                        with_window(|window| window.send(command));
                    }
                }
                None => {}
            }
            return LRESULT(0);
        }
        WM_TRAY_REQUEST => {
            with_window(TrayWindow::handle_requests);
            return LRESULT(0);
        }
        _ if msg != 0 && msg == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            if let Some(Err(e)) = with_window(|window| window.add()) {
                mosaico_core::log_warn!("The shell restarted, but {e}");
            }
            return LRESULT(0);
        }
        _ => {}
    }
    // SAFETY: DefWindowProcW is the default message handler for
    // messages not handled above. Required by the WNDPROC contract.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
