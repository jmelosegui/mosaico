use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};

use mosaico_core::Error;

use super::deleted;

#[test]
fn a_value_or_key_that_is_not_there_counts_as_deleted() {
    for status in [ERROR_SUCCESS, ERROR_FILE_NOT_FOUND] {
        // Act
        let result = deleted(status, "remove the autostart entry");

        // Assert
        assert!(result.is_ok(), "{status:?} gave {result:?}");
    }
}

#[test]
fn any_other_failure_says_what_was_being_done() {
    // Act
    let result = deleted(ERROR_ACCESS_DENIED, "remove the autostart entry");

    // Assert
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "remove the autostart entry",
                ..
            })
        ),
        "{result:?}"
    );
}
