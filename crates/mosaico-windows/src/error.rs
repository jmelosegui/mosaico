//! Wrapping native errors in the core error.

use mosaico_core::{Error, Result};

/// Turns the error of a failed OS call into [`Error::Platform`].
///
/// `mosaico_core::Error` is a foreign type here, so this crate cannot
/// implement `From` for `windows::core::Error` and the other native
/// errors. Each call site names what it was doing instead, which also
/// makes the message say more than the bare error code.
pub(crate) trait OrPlatform<T> {
    /// Wraps the error as a failed attempt to `operation`, such as
    /// "create a border window".
    fn or_platform(self, operation: &'static str) -> Result<T>;
}

impl<T, E> OrPlatform<T> for std::result::Result<T, E>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fn or_platform(self, operation: &'static str) -> Result<T> {
        self.map_err(|e| Error::platform(operation, e))
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::E_ACCESSDENIED;

    use super::*;

    #[test]
    fn a_native_error_says_what_was_being_done() {
        // Arrange
        let failed: windows::core::Result<()> = Err(E_ACCESSDENIED.into());

        // Act
        let error = failed.or_platform("open registry key").unwrap_err();

        // Assert
        assert!(
            matches!(
                error,
                Error::Platform {
                    operation: "open registry key",
                    ..
                }
            ),
            "{error:?}"
        );
        assert!(
            error
                .to_string()
                .starts_with("could not open registry key: "),
            "{error}"
        );
    }
}
