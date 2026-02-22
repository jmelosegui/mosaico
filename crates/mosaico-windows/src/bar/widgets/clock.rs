//! Clock widget — displays the current time.

use super::system;

/// Returns the current time formatted according to the given
/// strftime-style format string (e.g. `"%H:%M:%S"`).
pub fn text(format: &str) -> String {
    system::format_local_time(format)
}
