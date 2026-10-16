use windows::Win32::Foundation::E_HANDLE;

use super::*;

#[test]
fn a_system_without_corner_styles_is_reported_as_unsupported() {
    // Arrange -- Windows 10 does not know the attribute.
    let failed: windows::core::Result<()> = Err(E_INVALIDARG.into());

    // Act
    let result = corner_result(failed, "set the window corners");

    // Assert
    assert!(matches!(result, Err(Error::Unsupported(_))), "{result:?}");
}

#[test]
fn a_window_that_is_gone_is_a_platform_error() {
    // Arrange
    let failed: windows::core::Result<()> = Err(E_HANDLE.into());

    // Act
    let result = corner_result(failed, "set the window corners");

    // Assert
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "set the window corners",
                ..
            })
        ),
        "{result:?}"
    );
}
