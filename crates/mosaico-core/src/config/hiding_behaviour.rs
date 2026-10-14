use serde::{Deserialize, Serialize};

/// How windows are hidden when switching away from their workspace.
///
/// Each value is named after what the user sees, not after how a
/// platform does it. The names in `config.toml` (`"cloak"`, `"hide"`
/// and `"minimize"`) are also how the record of hidden windows is
/// written, so they must not change.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HidingBehaviour {
    /// The window becomes invisible but keeps its taskbar entry, and
    /// picking it there switches to its workspace. The desktop still
    /// reports it shown and not minimized. Recommended default.
    #[default]
    Cloak,
    /// The window is fully hidden, and its taskbar entry goes away until
    /// it is shown again. The desktop reports it not shown.
    Hide,
    /// The window is minimized: it keeps its taskbar entry, shown as
    /// minimized, and the application sees an ordinary minimize, which
    /// some react to, for example by pausing media. The desktop reports
    /// it minimized.
    Minimize,
}
