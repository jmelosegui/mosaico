//! Desktop events as one line of text, for people.

use std::fmt;

use super::DesktopEvent;

/// The event's name, then its fields as `name=value`, with ids printed
/// the way `debug state` prints them and text quoted.
impl fmt::Display for DesktopEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.kind().name())?;
        match self {
            Self::WindowManaged {
                window,
                monitor,
                workspace,
                class,
                title,
            } => write!(
                f,
                " window={window} monitor={monitor} workspace={} class={class:?} title={title:?}",
                workspace + 1
            ),
            Self::WindowUnmanaged { window } => write!(f, " window={window}"),
            Self::WindowMoved {
                window,
                monitor,
                workspace,
            } => write!(
                f,
                " window={window} monitor={monitor} workspace={}",
                workspace + 1
            ),
            Self::WindowTitleChanged { window, title } => {
                write!(f, " window={window} title={title:?}")
            }
            Self::WindowFloatingChanged { window, floating } => {
                write!(f, " window={window} floating={floating}")
            }
            Self::FocusChanged { window, monitor } => match window {
                Some(window) => write!(f, " window={window} monitor={monitor}"),
                None => write!(f, " window=none monitor={monitor}"),
            },
            Self::WorkspaceChanged { monitor, workspace } => {
                write!(f, " monitor={monitor} workspace={}", workspace + 1)
            }
            Self::LayoutChanged {
                monitor,
                workspace,
                layout,
                monocle,
            } => write!(
                f,
                " monitor={monitor} workspace={} layout={} monocle={monocle}",
                workspace + 1,
                layout.name()
            ),
            Self::MonitorAdded { monitor } | Self::MonitorRemoved { monitor } => {
                write!(f, " monitor={monitor}")
            }
            Self::Paused { paused } => write!(f, " paused={paused}"),
            Self::ConfigReloaded | Self::Lagged => Ok(()),
        }
    }
}

impl DesktopEvent {
    /// The event as one line of JSON, as subscribers receive it.
    pub fn to_json(&self) -> String {
        // Every field is a plain value, so encoding cannot fail.
        serde_json::to_string(self).unwrap_or_default()
    }
}
