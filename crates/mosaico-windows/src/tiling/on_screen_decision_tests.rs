//! Which windows are on screen, from what the walk reads of them: each
//! read that rules a window out, on its own.

use super::*;

/// What a test has a window read.
struct Seen {
    visible: bool,
    minimized: bool,
    ex_style: u32,
    process: Option<u32>,
    cloaked: bool,
    class: &'static str,
}

impl Reads for Seen {
    fn visible(&self) -> bool {
        self.visible
    }

    fn minimized(&self) -> bool {
        self.minimized
    }

    fn ex_style(&self) -> u32 {
        self.ex_style
    }

    fn process(&self) -> Option<u32> {
        self.process
    }

    fn cloaked(&self) -> bool {
        self.cloaked
    }

    fn class_name(&self) -> String {
        self.class.to_string()
    }
}

/// The process the daemon runs as, in these tests.
const DAEMON: u32 = 7;

/// A browser window of another process, shown.
fn browser() -> Seen {
    Seen {
        visible: true,
        minimized: false,
        ex_style: 0,
        process: Some(42),
        cloaked: false,
        class: "Chrome_WidgetWin_1",
    }
}

#[test]
fn a_shown_application_window_is_on_screen() {
    // Arrange -- layered, as many are, but taking the pointer.
    let window = Seen {
        ex_style: WS_EX_LAYERED.0,
        ..browser()
    };

    // Act / Assert
    assert!(on_screen(&window, DAEMON));
}

#[test]
fn a_hidden_window_is_not_on_screen() {
    // Arrange
    let window = Seen {
        visible: false,
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}

#[test]
fn a_minimized_window_is_not_on_screen() {
    // Arrange
    let window = Seen {
        minimized: true,
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}

#[test]
fn a_window_the_pointer_goes_through_is_not_on_screen() {
    // Arrange -- an overlay over a monitor, layered and transparent.
    let window = Seen {
        ex_style: WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0,
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}

#[test]
fn a_window_of_the_daemon_is_not_on_screen() {
    // Arrange -- a bar or a border.
    let window = Seen {
        process: Some(DAEMON),
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}

#[test]
fn a_cloaked_window_is_not_on_screen() {
    // Arrange -- as on another virtual desktop, or hidden by the tiler.
    let window = Seen {
        cloaked: true,
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}

#[test]
fn a_surface_of_the_shell_is_not_on_screen() {
    // Arrange -- the desktop, over every monitor.
    let window = Seen {
        class: "Progman",
        ..browser()
    };

    // Act / Assert
    assert!(!on_screen(&window, DAEMON));
}
