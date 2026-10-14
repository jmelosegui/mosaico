use std::io::{BufReader, Cursor};

use super::*;
use crate::ipc::{Command, Request};

#[test]
fn a_message_round_trips_as_one_line() {
    // Arrange
    let request = Request::new(Command::Status);
    let mut wire = Vec::new();

    // Act
    write_message(&mut wire, &request).unwrap();
    let back: Request = read_message(&mut BufReader::new(Cursor::new(&wire))).unwrap();

    // Assert
    assert_eq!(back, request);
    assert_eq!(wire.iter().filter(|&&b| b == b'\n').count(), 1);
    assert_eq!(wire.last(), Some(&b'\n'));
}

#[test]
fn newlines_inside_strings_do_not_split_a_message() {
    // Arrange
    let text = "line one\nline two".to_string();
    let mut wire = Vec::new();

    // Act
    write_message(&mut wire, &text).unwrap();
    let back: String = read_message(&mut BufReader::new(Cursor::new(&wire))).unwrap();

    // Assert
    assert_eq!(back, text);
}

#[test]
fn consecutive_messages_are_read_one_at_a_time() {
    // Arrange
    let mut wire = Vec::new();
    write_message(&mut wire, &1u32).unwrap();
    write_message(&mut wire, &2u32).unwrap();
    let mut reader = BufReader::new(Cursor::new(wire));

    // Act / Assert
    assert_eq!(read_message::<u32>(&mut reader).unwrap(), 1);
    assert_eq!(read_message::<u32>(&mut reader).unwrap(), 2);
}

#[test]
fn a_stream_that_ends_mid_message_is_closed() {
    // Arrange -- the daemon died after writing half a reply.
    let mut reader = BufReader::new(Cursor::new(b"{\"status\":".to_vec()));

    // Act
    let result = read_message::<serde_json::Value>(&mut reader);

    // Assert
    assert!(matches!(result, Err(IpcError::Closed)));
}

#[test]
fn an_empty_stream_is_closed() {
    // Arrange
    let mut reader = BufReader::new(Cursor::new(Vec::new()));

    // Act
    let result = read_message::<serde_json::Value>(&mut reader);

    // Assert
    assert!(matches!(result, Err(IpcError::Closed)));
}

#[test]
fn an_oversized_message_is_rejected_without_reading_it_all() {
    // Arrange
    let mut wire = vec![b'"'; MAX_MESSAGE_BYTES + 10];
    wire.push(b'\n');
    let mut reader = BufReader::new(Cursor::new(wire));

    // Act
    let result = read_message::<String>(&mut reader);

    // Assert
    assert!(matches!(result, Err(IpcError::TooLarge)));
}

#[test]
fn a_message_of_exactly_the_limit_is_accepted() {
    // Arrange -- a JSON string of MAX_MESSAGE_BYTES bytes, quotes included.
    let body = "a".repeat(MAX_MESSAGE_BYTES - 2);
    let mut wire = Vec::new();
    write_message(&mut wire, &body).unwrap();

    // Act
    let back: String = read_message(&mut BufReader::new(Cursor::new(wire))).unwrap();

    // Assert
    assert_eq!(back.len(), MAX_MESSAGE_BYTES - 2);
}

#[test]
fn writing_an_oversized_message_fails() {
    // Arrange
    let body = "a".repeat(MAX_MESSAGE_BYTES);
    let mut wire = Vec::new();

    // Act
    let result = write_message(&mut wire, &body);

    // Assert
    assert!(matches!(result, Err(IpcError::TooLarge)));
    assert!(wire.is_empty(), "nothing goes on the wire");
}

#[test]
fn invalid_json_is_malformed() {
    // Arrange
    let mut reader = BufReader::new(Cursor::new(b"not json\n".to_vec()));

    // Act
    let result = read_message::<serde_json::Value>(&mut reader);

    // Assert
    assert!(matches!(result, Err(IpcError::Malformed(_))));
}
