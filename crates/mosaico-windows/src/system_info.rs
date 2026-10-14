//! Machine readings for the bar's system widgets, from Win32.

use std::mem;

use mosaico_core::Error;
use mosaico_core::bar::{CpuTimes, LocalTime, NowPlaying, SystemInfo};
use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::SystemInformation::{
    GetLocalTime, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};
use windows::Win32::System::Threading::GetSystemTimes;

use crate::error::OrPlatform;

/// Reads processor time, memory load and the local time from Win32.
pub struct Win32SystemInfo;

impl SystemInfo for Win32SystemInfo {
    fn cpu_times(&self) -> mosaico_core::Result<CpuTimes> {
        let mut idle = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();

        // SAFETY: GetSystemTimes fills the FILETIME structs with system-wide
        // idle, kernel, and user time counters. All pointers are valid stack refs.
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }
            .or_platform("read the processor times")?;

        // The counters are 100 ns ticks since boot, and kernel time
        // includes the idle time, so kernel plus user is the total.
        Ok(CpuTimes {
            idle: ticks(idle),
            total: ticks(kernel) + ticks(user),
        })
    }

    fn memory_usage(&self) -> mosaico_core::Result<u32> {
        let mut status = MEMORYSTATUSEX {
            dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };

        // SAFETY: GlobalMemoryStatusEx fills the MEMORYSTATUSEX struct with
        // current memory statistics. dwLength must be set before the call.
        unsafe { GlobalMemoryStatusEx(&mut status) }.or_platform("read the memory status")?;

        Ok(status.dwMemoryLoad)
    }

    fn now_playing(&self) -> mosaico_core::Result<Option<NowPlaying>> {
        // The GSMTC `RequestAsync().get()` call blocks forever in the
        // daemon process (no GUI message pump), so the media session is
        // not read until a non-blocking query is added.
        Err(Error::Unsupported("reading the media that is playing"))
    }

    fn local_time(&self) -> LocalTime {
        // SAFETY: GetLocalTime returns the current local time as a
        // SYSTEMTIME struct. It is always safe to call and never fails.
        let st: SYSTEMTIME = unsafe { GetLocalTime() };

        LocalTime {
            year: st.wYear,
            month: narrow(st.wMonth),
            day: narrow(st.wDay),
            weekday: narrow(st.wDayOfWeek),
            hour: narrow(st.wHour),
            minute: narrow(st.wMinute),
            second: narrow(st.wSecond),
        }
    }
}

/// A FILETIME as one 64-bit count.
fn ticks(ft: FILETIME) -> u64 {
    u64::from(ft.dwLowDateTime) | (u64::from(ft.dwHighDateTime) << 32)
}

/// A SYSTEMTIME field as a byte. They always fit; one that did not would
/// show as out of range instead of wrapping.
fn narrow(value: u16) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}
