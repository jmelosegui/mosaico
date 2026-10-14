//! The time the clock and date widgets show, and how they format it.

/// A wall-clock reading in the user's time zone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalTime {
    /// Year, such as 2026.
    pub year: u16,
    /// Month, from 1 for January to 12.
    pub month: u8,
    /// Day of the month, from 1.
    pub day: u8,
    /// Day of the week, from 0 for Sunday to 6.
    pub weekday: u8,
    /// Hour, from 0 to 23.
    pub hour: u8,
    /// Minute, from 0 to 59.
    pub minute: u8,
    /// Second, from 0 to 59.
    pub second: u8,
}

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const MONTHS: [&str; 12] = [
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

impl LocalTime {
    /// Formats the time with a strftime-like `format`, such as
    /// `"%H:%M:%S"`.
    ///
    /// Supports `%H` (hour), `%M` (minute), `%S` (second), `%A` (weekday),
    /// `%d` (day), `%B` (month name), `%Y` (year) and `%%` (a literal %).
    /// Anything else after a `%` is kept as written. A weekday or month
    /// out of range shows as `???`.
    pub fn format(&self, format: &str) -> String {
        let mut result = String::with_capacity(format.len() + 16);
        let mut chars = format.chars();

        while let Some(c) = chars.next() {
            if c != '%' {
                result.push(c);
                continue;
            }
            match chars.next() {
                Some('H') => result.push_str(&format!("{:02}", self.hour)),
                Some('M') => result.push_str(&format!("{:02}", self.minute)),
                Some('S') => result.push_str(&format!("{:02}", self.second)),
                Some('A') => result.push_str(name(&WEEKDAYS, usize::from(self.weekday))),
                Some('d') => result.push_str(&format!("{:02}", self.day)),
                Some('B') => {
                    let month = usize::from(self.month.wrapping_sub(1));
                    result.push_str(name(&MONTHS, month));
                }
                Some('Y') => result.push_str(&self.year.to_string()),
                Some('%') => result.push('%'),
                Some(other) => {
                    result.push('%');
                    result.push(other);
                }
                None => result.push('%'),
            }
        }

        result
    }
}

/// The name at `index`, or `???` when there is none.
fn name(names: &[&'static str], index: usize) -> &'static str {
    names.get(index).copied().unwrap_or("???")
}

#[cfg(test)]
#[path = "tests/time_tests.rs"]
mod tests;
