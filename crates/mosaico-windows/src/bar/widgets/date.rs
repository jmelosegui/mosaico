//! Date widget — displays the current date.

use super::system;

/// Returns the current date formatted according to the given
/// strftime-style format string (e.g. `"%A %d %B %Y"`).
pub fn text(format: &str) -> String {
    system::format_local_time(format)
}
