//! Shared system utilities used by multiple widgets.

use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::System::SystemInformation::GetLocalTime;

/// Formats the current local time using a strftime-like format string.
///
/// Supports: `%H` (hour), `%M` (minute), `%S` (second), `%A` (weekday),
/// `%d` (day), `%B` (month name), `%Y` (year), `%%` (literal %).
pub fn format_local_time(fmt: &str) -> String {
    let st: SYSTEMTIME = unsafe { GetLocalTime() };

    let weekdays = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let mut result = String::with_capacity(fmt.len() + 16);
    let mut chars = fmt.chars();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('H') => result.push_str(&format!("{:02}", st.wHour)),
                Some('M') => result.push_str(&format!("{:02}", st.wMinute)),
                Some('S') => result.push_str(&format!("{:02}", st.wSecond)),
                Some('A') => {
                    let day = st.wDayOfWeek as usize;
                    result.push_str(weekdays.get(day).unwrap_or(&"???"));
                }
                Some('d') => result.push_str(&format!("{:02}", st.wDay)),
                Some('B') => {
                    let mon = st.wMonth.wrapping_sub(1) as usize;
                    result.push_str(months.get(mon).unwrap_or(&"???"));
                }
                Some('Y') => result.push_str(&format!("{}", st.wYear)),
                Some('%') => result.push('%'),
                Some(other) => {
                    result.push('%');
                    result.push(other);
                }
                None => result.push('%'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
