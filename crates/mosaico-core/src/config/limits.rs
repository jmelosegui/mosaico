//! The ranges numbers in the config files are kept to. Loading a file
//! clamps them, but for the log size, and the settings window offers no
//! more than these.

/// The widest gap between windows, in pixels.
pub const GAP_MAX: i32 = 200;

/// The least share of a split the first window gets.
pub const RATIO_MIN: f64 = 0.1;

/// The greatest share of a split the first window gets.
pub const RATIO_MAX: f64 = 0.9;

/// The widest border, in pixels.
pub const BORDER_WIDTH_MAX: i32 = 32;

/// The longest a window transition takes, in milliseconds.
pub const DURATION_MAX_MS: i32 = 1000;

/// The sizes, in megabytes, a log file may grow to before it is rotated.
pub const LOG_FILE_MB: (u64, u64) = (1, 1024);
