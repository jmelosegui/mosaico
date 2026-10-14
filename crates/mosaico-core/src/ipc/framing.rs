//! How messages are delimited on an IPC stream.
//!
//! Every transport frames messages the same way: one compact JSON value
//! followed by a newline. Compact JSON never contains a raw newline (one
//! inside a string is escaped), so the newline can only be the end of a
//! message. Keeping this here means a client on one transport and a
//! daemon on another can never disagree about where a message ends.

use std::io::{BufRead, Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::IpcError;

/// Largest message, in bytes, not counting the newline.
///
/// Generous enough for the `debug state` dump of a busy desktop, small
/// enough that a peer sending garbage cannot make the other side buffer
/// without bound.
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Writes one message and flushes it.
pub fn write_message<T: Serialize>(writer: &mut impl Write, message: &T) -> Result<(), IpcError> {
    let mut bytes = serde_json::to_vec(message).map_err(IpcError::Malformed)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(IpcError::TooLarge);
    }
    bytes.push(b'\n');
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

/// Reads one message.
///
/// Fails with [`IpcError::Closed`] when the peer goes away before a
/// whole message arrives, which is what a client sees if the daemon
/// exits mid-request.
pub fn read_message<T: DeserializeOwned>(reader: &mut impl BufRead) -> Result<T, IpcError> {
    let mut bytes = Vec::new();
    let limit = MAX_MESSAGE_BYTES as u64 + 1;
    reader.by_ref().take(limit).read_until(b'\n', &mut bytes)?;

    if bytes.last() != Some(&b'\n') {
        // Either the limit ran out before a newline, or the stream ended.
        return Err(if bytes.len() > MAX_MESSAGE_BYTES {
            IpcError::TooLarge
        } else {
            IpcError::Closed
        });
    }
    bytes.pop();
    serde_json::from_slice(&bytes).map_err(IpcError::Malformed)
}

#[cfg(test)]
#[path = "tests/framing_tests.rs"]
mod tests;
