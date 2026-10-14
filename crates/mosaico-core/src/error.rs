//! The error type shared by core and every platform contract.

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::instance::InstanceError;
use crate::ipc::IpcError;

/// The result of an operation that fails with an [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why an operation failed.
///
/// Platform contracts in core return this, so callers can tell the
/// failure domains apart, and say something useful, without knowing
/// which platform they run on. Platform crates keep their native error
/// as the source of [`Error::Platform`], see [`Error::platform`].
#[derive(Debug)]
pub enum Error {
    /// A config file could not be read, parsed or validated.
    Config {
        /// The file, when the failure is about one.
        path: Option<PathBuf>,
        /// What is wrong, in words for the user.
        message: String,
    },
    /// Talking to the daemon failed.
    Ipc(IpcError),
    /// The instance settings are invalid.
    Instance(InstanceError),
    /// A daemon of this instance is already running, so another one
    /// must not start.
    AlreadyRunning,
    /// A call into the operating system, or the platform layer around it,
    /// failed.
    Platform {
        /// What was being done, such as "create the border window".
        operation: &'static str,
        /// The error the OS reported, or what went wrong in the platform
        /// layer, such as a thread it needs that exited.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Reading or writing a file or stream failed.
    Io(io::Error),
    /// A network request failed.
    Network(String),
    /// The platform cannot do this at all.
    Unsupported(&'static str),
}

impl Error {
    /// A config error, about `path` when there is one.
    pub fn config(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self::Config {
            path,
            message: message.into(),
        }
    }

    /// A failure while doing `operation`, in an OS call or the platform
    /// layer around it.
    ///
    /// Platform crates use this to wrap their native errors, since they
    /// cannot implement `From` for a foreign error type.
    pub fn platform(
        operation: &'static str,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Platform {
            operation,
            source: source.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config {
                path: Some(path),
                message,
            } => write!(f, "{}: {message}", path.display()),
            Self::Config {
                path: None,
                message,
            } => f.write_str(message),
            Self::Ipc(e) => write!(f, "{e}"),
            Self::Instance(e) => write!(f, "{e}"),
            Self::AlreadyRunning => f.write_str("Mosaico daemon is already running."),
            Self::Platform { operation, source } => {
                write!(f, "could not {operation}: {source}")
            }
            Self::Io(e) => write!(f, "{e}"),
            Self::Network(message) => write!(f, "network request failed: {message}"),
            Self::Unsupported(what) => write!(f, "{what} is not supported on this platform"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ipc(e) => Some(e),
            Self::Instance(e) => Some(e),
            Self::Platform { source, .. } => Some(source.as_ref()),
            Self::Io(e) => Some(e),
            Self::Config { .. }
            | Self::AlreadyRunning
            | Self::Network(_)
            | Self::Unsupported(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<IpcError> for Error {
    fn from(e: IpcError) -> Self {
        Self::Ipc(e)
    }
}

impl From<InstanceError> for Error {
    fn from(e: InstanceError) -> Self {
        Self::Instance(e)
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
