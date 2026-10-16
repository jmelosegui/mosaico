use mosaico_core::Command;
use mosaico_core::ipc::DesktopSnapshot;

use crate::commands::context::{Context, Exit, say};
use crate::commands::request;

/// Prints the daemon's internal workspace state.
///
/// The daemon's own text is printed when it sends one, so a newer daemon
/// is shown the way it describes itself. The payload is rendered here
/// only when there is no text.
pub fn execute(cx: &mut Context) -> Exit {
    let response = match request::send(cx, &Command::Inspect) {
        Ok(response) => response,
        Err(exit) => return exit,
    };
    if let Some(message) = &response.message {
        say!(cx, "{message}");
    } else if let Some(snapshot) = response.data::<DesktopSnapshot>() {
        say!(cx, "{snapshot}");
    }
    Exit::Success
}

#[cfg(test)]
mod tests {
    use mosaico_core::ipc::Response;

    use super::*;
    use crate::commands::test_support::Harness;

    fn snapshot() -> DesktopSnapshot {
        DesktopSnapshot {
            monitors: Vec::new(),
            disconnected: Vec::new(),
            displaced_windows: 0,
            daemon_elevated: false,
            focused_monitor: 0,
            focused_window: None,
        }
    }

    #[test]
    fn prints_the_text_the_daemon_sends() {
        // Arrange -- the text differs from what the payload renders as,
        // to show which one is printed.
        let mut h = Harness::new("state-text");
        h.world.daemon_up(1);
        let reply = Response::ok_with_message("the daemon's own text").with_data(&snapshot());
        h.world.reply(Ok(reply));

        // Act
        let exit = h.run(execute);

        // Assert
        assert_eq!(exit, Exit::Success);
        assert_eq!(h.out(), "the daemon's own text\n");
    }

    #[test]
    fn renders_the_state_when_the_daemon_sends_no_text() {
        // Arrange
        let mut h = Harness::new("state-data");
        h.world.daemon_up(1);
        h.world.reply(Ok(Response::ok().with_data(&snapshot())));

        // Act
        let exit = h.run(execute);

        // Assert
        assert_eq!(exit, Exit::Success);
        assert_eq!(h.out(), format!("{}\n", snapshot()));
    }

    #[test]
    fn fails_when_no_daemon_is_running() {
        // Arrange
        let mut h = Harness::new("state-down");

        // Act
        let exit = h.run(execute);

        // Assert
        assert_eq!(exit, Exit::Failure);
        assert_eq!(h.err(), "Mosaico is not running.\n");
    }
}
