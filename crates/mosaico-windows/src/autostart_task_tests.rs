use windows::Win32::Foundation::{E_ACCESSDENIED, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};
use windows::core::HRESULT;

use super::is_missing;

#[test]
fn a_task_that_is_not_found_is_missing() {
    // Act
    let missing = is_missing(HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0));

    // Assert
    assert!(missing);
}

#[test]
fn other_failures_are_not_a_missing_task() {
    // Arrange
    let failures = [E_ACCESSDENIED, HRESULT::from_win32(ERROR_PATH_NOT_FOUND.0)];

    for failure in failures {
        // Act
        let missing = is_missing(failure);

        // Assert
        assert!(!missing, "{failure:?}");
    }
}
