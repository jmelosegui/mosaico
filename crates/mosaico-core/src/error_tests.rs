use std::error::Error as _;
use std::io;
use std::path::PathBuf;

use crate::ipc::IpcError;

use super::Error;

#[test]
fn a_config_error_names_the_file() {
    // Arrange
    let error = Error::config(Some(PathBuf::from("config.toml")), "unknown key `gap`");

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(text, "config.toml: unknown key `gap`");
}

#[test]
fn a_platform_error_says_what_was_being_done_and_keeps_the_cause() {
    // Arrange
    let cause = io::Error::other("access denied");

    // Act
    let error = Error::platform("create the border window", cause);

    // Assert
    assert_eq!(
        error.to_string(),
        "could not create the border window: access denied"
    );
    assert_eq!(
        error.source().map(ToString::to_string).as_deref(),
        Some("access denied")
    );
}

#[test]
fn ipc_errors_convert_with_question_mark_and_can_still_be_matched() {
    // Arrange
    fn send() -> Result<(), IpcError> {
        Err(IpcError::NotRunning)
    }
    fn status() -> crate::Result<()> {
        send()?;
        Ok(())
    }

    // Act
    let result = status();

    // Assert
    assert!(matches!(result, Err(Error::Ipc(IpcError::NotRunning))));
}

#[test]
fn errors_cross_threads() {
    // Arrange
    fn assert_send_sync<T: Send + Sync>() {}

    // Act / Assert
    assert_send_sync::<Error>();
}
