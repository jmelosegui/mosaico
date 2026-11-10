use std::fs::File;
use std::os::windows::io::FromRawHandle;

use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows::Win32::System::Pipes::{CreateNamedPipeW, PIPE_TYPE_BYTE, PIPE_WAIT};

use mosaico_core::ipc::IpcClient;

use super::super::{PipeClient, PipeServer};
use super::*;

const USER: &str = "S-1-5-21-1-2-3-1001";

/// A pipe name no daemon uses, unique to this test.
fn test_pipe(test: &str) -> String {
    format!(
        r"\\.\pipe\mosaico-test-security-{test}-{}",
        std::process::id()
    )
}

/// One instance of the pipe `name`, created with the descriptor `text`.
fn pipe_with(name: &str, text: &str) -> File {
    let security = PipeSecurity::from_text(text).unwrap();
    let attributes = security.attributes();
    // SAFETY: Creates one pipe instance with the descriptor above, which
    // outlives the call; the handle is checked below.
    let handle = unsafe {
        CreateNamedPipeW(
            &HSTRING::from(name),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_WAIT,
            1,
            4096,
            4096,
            0,
            Some(&attributes),
        )
    };
    assert_ne!(
        handle,
        INVALID_HANDLE_VALUE,
        "{}",
        io::Error::last_os_error()
    );
    // SAFETY: The handle was just created and nothing else owns it.
    unsafe { File::from_raw_handle(handle.0) }
}

#[test]
fn the_descriptor_lets_in_only_the_user_and_the_system_at_the_daemons_level() {
    // Act
    let text = descriptor_text(USER, "S-1-16-12288");

    // Assert
    assert_eq!(
        text,
        "O:S-1-5-21-1-2-3-1001D:P(D;;GA;;;NU)(A;;GA;;;SY)(A;;GA;;;S-1-5-21-1-2-3-1001)\
         S:(ML;;NWNR;;;S-1-16-12288)"
    );
}

#[test]
fn a_pipe_the_user_owns_at_medium_or_above_is_trusted() {
    // Arrange
    let cases = [
        format!("O:{USER}S:(ML;;NWNR;;;ME)"),
        format!("O:{USER}S:(ML;;NWNR;;;HI)"),
        format!("O:{USER}G:{USER}S:(ML;;NW;;;S-1-16-8192)"),
        // No label is medium, as the system reads it.
        format!("O:{USER}"),
    ];

    // Act / Assert
    for text in cases {
        assert!(trusts(&text, USER), "{text}");
    }
}

#[test]
fn a_pipe_of_another_owner_or_below_medium_is_not_trusted() {
    // Arrange
    let cases = [
        "O:S-1-5-21-1-2-3-1002S:(ML;;NWNR;;;ME)".to_string(),
        "O:BAS:(ML;;NWNR;;;HI)".to_string(),
        format!("O:{USER}S:(ML;;NW;;;LW)"),
        format!("O:{USER}S:(ML;;NW;;;S-1-16-0)"),
        format!("O:{USER}S:(ML;;NW;;;garbage)"),
        "S:(ML;;NW;;;ME)".to_string(),
    ];

    // Act / Assert
    for text in cases {
        assert!(!trusts(&text, USER), "{text}");
    }
}

#[test]
fn the_daemons_pipe_is_owned_by_the_user_and_labelled_at_its_level() {
    // Arrange
    let name = test_pipe("owner");
    let mut server = PipeServer::new(&name);

    // Act
    let instance = server.create_instance().unwrap();
    let text = owner_and_label(HANDLE(instance.as_raw_handle())).unwrap();

    // Assert
    let user = token::current_user().unwrap();
    let level = integrity(&token::current_integrity().unwrap()).unwrap();
    assert_eq!(owner(&text), Some(user.as_str()), "{text}");
    assert_eq!(label(&text), Some(level), "{text}");
}

#[test]
fn the_daemon_does_not_join_a_pipe_someone_else_created_first() {
    // Arrange -- another program already holds the name.
    let name = test_pipe("squatted");
    let user = token::current_user().unwrap();
    let _squatter = pipe_with(&name, &format!("O:{user}D:(A;;GA;;;WD)"));

    // Act
    let created = PipeServer::new(&name).create_instance();

    // Assert
    assert!(
        created.is_err(),
        "the daemon joined a pipe it did not create"
    );
}

#[test]
fn a_client_refuses_a_pipe_labelled_below_medium() {
    // Arrange -- a pipe of the user's own, made by a low-trust program.
    let name = test_pipe("low");
    let user = token::current_user().unwrap();
    let _low = pipe_with(&name, &format!("O:{user}D:(A;;GA;;;WD)S:(ML;;NW;;;LW)"));

    // Act
    let connected = PipeClient::new(&name).connect();

    // Assert
    assert!(
        matches!(connected, Err(IpcError::Untrusted)),
        "{:?}",
        connected.err()
    );
}

#[test]
fn a_client_connects_to_the_daemons_own_pipe() {
    // Arrange
    let name = test_pipe("own");
    let mut server = PipeServer::new(&name);
    let _instance = server.create_instance().unwrap();

    // Act
    let connected = PipeClient::new(&name).connect();

    // Assert
    assert!(connected.is_ok(), "{:?}", connected.err());
}
