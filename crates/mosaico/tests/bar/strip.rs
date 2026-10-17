//! The bar the bar click tests press and scroll: the test daemon's bar on
//! the primary monitor, found among the windows on screen by its class
//! and its process.

use std::thread;
use std::time::{Duration, Instant};

use super::{bar_win32, win32};

/// The class of a bar's window.
const CLASS: &str = "MosaicoBar";

/// How long the daemon is given to show its bar, taking the pointer.
const SHOWN: Duration = Duration::from_secs(10);

/// A bar's window, the process that made it, and where it is, in the
/// physical pixels of the virtual desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub hwnd: isize,
    process: u32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bar {
    /// The point half way down the bar at `x`.
    pub fn at(&self, x: i32) -> (i32, i32) {
        (x, (self.top + self.bottom) / 2)
    }

    /// Whether the bar takes the pointer, rather than letting it through to
    /// the windows under it, as it does while one of them is maximized or
    /// fullscreen.
    fn takes_pointer(&self) -> bool {
        // SAFETY: a read-only query, which gives 0 for a window that is gone.
        let style = unsafe { bar_win32::GetWindowLongW(self.hwnd, bar_win32::GWL_EXSTYLE) };
        style != 0 && style & bar_win32::WS_EX_TRANSPARENT == 0
    }
}

/// The bar of the daemon whose process is `daemon` on the primary monitor,
/// once the daemon shows it there and it takes the pointer. Fails the test
/// when that takes too long, and when the bar of another process shows
/// there, as that of a daemon an interrupted run left behind.
pub fn primary(daemon: u32) -> Bar {
    let deadline = Instant::now() + SHOWN;
    loop {
        let bars: Vec<Bar> = (visible_bars().into_iter())
            .filter(on_the_primary_monitor)
            .collect();
        if let Some(other) = bars.iter().find(|bar| bar.process != daemon) {
            panic!(
                "the primary monitor shows a bar the test daemon did not make, \
                 {other:?}: is a daemon of an earlier run still up?"
            );
        }
        if let Some(&bar) = bars.first().filter(|bar| bar.takes_pointer()) {
            return bar;
        }
        let why = match bars.first() {
            Some(_) => "the bar lets clicks through: is a window maximized or fullscreen there?",
            None => "the test daemon shows no bar on the primary monitor",
        };
        assert!(Instant::now() < deadline, "{why}");
        thread::sleep(Duration::from_millis(100));
    }
}

/// Every visible window of the bars' class.
fn visible_bars() -> Vec<Bar> {
    unsafe extern "system" fn each(hwnd: isize, lparam: isize) -> i32 {
        // SAFETY: lparam is the vector EnumWindows is given below, which
        // outlives the enumeration, and nothing else touches it meanwhile.
        let bars = unsafe { &mut *(lparam as *mut Vec<Bar>) };
        let mut name = [0u16; 64];
        let mut r = bar_win32::Rect::default();
        let mut process = 0;
        // SAFETY: read-only queries on a handle EnumWindows gave, and the
        // name, the rect and the process id are written on the stack, the
        // name no longer than the call is told.
        let is_bar = unsafe {
            let length = bar_win32::GetClassNameW(hwnd, name.as_mut_ptr(), 64);
            String::from_utf16_lossy(&name[..length as usize]) == CLASS
                && bar_win32::IsWindowVisible(hwnd) != 0
                && bar_win32::GetWindowRect(hwnd, &mut r) != 0
                && bar_win32::GetWindowThreadProcessId(hwnd, &mut process) != 0
        };
        if is_bar {
            let (left, top, right, bottom) = (r.left, r.top, r.right, r.bottom);
            bars.push(Bar {
                hwnd,
                process,
                left,
                top,
                right,
                bottom,
            });
        }
        1
    }
    let mut bars: Vec<Bar> = Vec::new();
    // SAFETY: the callback only touches `bars`, during the call.
    unsafe { bar_win32::EnumWindows(each, &mut bars as *mut Vec<Bar> as isize) };
    bars
}

/// Whether `bar` is on the primary monitor, the one the origin of the
/// virtual desktop is on.
fn on_the_primary_monitor(bar: &Bar) -> bool {
    let origin = win32::Point { x: 0, y: 0 };
    // SAFETY: read-only queries, and a window that is gone is on no monitor.
    unsafe {
        let primary = bar_win32::MonitorFromPoint(origin, bar_win32::MONITOR_DEFAULTTOPRIMARY);
        let on = bar_win32::MonitorFromWindow(bar.hwnd, bar_win32::MONITOR_DEFAULTTONULL);
        on != 0 && on == primary
    }
}
