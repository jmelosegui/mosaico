use super::*;

#[test]
fn the_desktop_and_the_taskbars_belong_to_the_shell() {
    // Arrange
    let classes = [
        "Progman",
        "WorkerW",
        "Shell_TrayWnd",
        "Shell_SecondaryTrayWnd",
    ];

    // Act
    let shell: Vec<bool> = classes.iter().map(|class| is_shell(class)).collect();

    // Assert
    assert_eq!(shell, [true; 4]);
}

#[test]
fn an_application_window_does_not_belong_to_the_shell() {
    // Arrange -- a browser's, and the frame of a store application.
    let classes = ["Chrome_WidgetWin_1", "ApplicationFrameWindow"];

    // Act
    let shell: Vec<bool> = classes.iter().map(|class| is_shell(class)).collect();

    // Assert
    assert_eq!(shell, [false; 2]);
}

#[test]
fn only_a_layered_window_that_is_also_transparent_lets_the_pointer_through() {
    // Arrange
    let (layered, transparent) = (WS_EX_LAYERED.0, WS_EX_TRANSPARENT.0);

    // Act
    let through = [layered | transparent, layered, transparent, 0].map(lets_pointer_through);

    // Assert
    assert_eq!(through, [true, false, false, false]);
}

#[test]
fn no_window_listed_belongs_to_the_shell_or_to_this_process() {
    // Arrange -- the desktop as it is, which this only reads. A window may
    // be hidden or minimized meanwhile, but not change class or process.
    let windows = windows_on_screen();

    // Act
    let wrong: Vec<WindowId> = windows
        .into_iter()
        .filter(|&id| {
            let win = Window::from_id(id);
            win.process_id() == Some(std::process::id())
                || win.class().is_ok_and(|class| is_shell(&class))
        })
        .collect();

    // Assert
    assert_eq!(wrong, []);
}
