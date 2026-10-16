use serde::{Deserialize, Serialize};

/// How windows are hidden when switching away from their workspace.
///
/// Each value is named after what the user sees, not after how a
/// platform does it. Which of them a platform has, and what hiding a
/// window each way reports back, is up to the platform: see
/// [`WindowControl::hiding_events`](crate::WindowControl::hiding_events).
/// When the platform does not have the one configured, the tiler uses
/// another and says so in the log, see
/// [`HidingSupport::choose`](crate::HidingSupport::choose).
///
/// The names in `config.toml` (`"cloak"`, `"hide"` and `"minimize"`)
/// are also how the record of hidden windows is written, so they must
/// not change.
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

impl HidingBehaviour {
    /// The value as it is written in `config.toml`, such as `"cloak"`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Cloak => "cloak",
            Self::Hide => "hide",
            Self::Minimize => "minimize",
        }
    }
}
