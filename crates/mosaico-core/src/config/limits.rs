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

/// The heights, in pixels, the bar may have.
pub const BAR_HEIGHT: (i32, i32) = (16, 96);

/// The sizes, in pixels, the bar's text may have.
pub const BAR_FONT_SIZE: (i32, i32) = (8, 48);

/// The padding at the bar's edges, in pixels.
pub const BAR_PADDING: (i32, i32) = (0, 64);

/// The padding inside a widget's pill, in pixels.
pub const BAR_PILL_PADDING: (i32, i32) = (0, 32);

/// The corner radius of a pill, in pixels.
pub const BAR_PILL_RADIUS: (i32, i32) = (0, 32);

/// The border width of a pill, in pixels.
pub const BAR_PILL_BORDER: (i32, i32) = (0, 8);

/// The gap between pills, in pixels.
pub const BAR_ITEM_GAP: (i32, i32) = (0, 32);

/// The gap between workspace numbers, in pixels.
pub const BAR_WORKSPACE_GAP: (i32, i32) = (0, 16);

/// The opacity of the bar's background, as a percentage.
pub const BAR_OPACITY: (i32, i32) = (0, 100);
