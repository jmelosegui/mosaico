use std::sync::mpsc;

use mosaico_core::ipc::Command;

use crate::ipc::PipeServer;

use super::daemon_types::{DaemonMsg, ResponseSender};

/// Accepts IPC connections in a loop and forwards commands to the
/// main daemon thread. Runs on a dedicated thread.
pub(super) fn ipc_loop(tx: mpsc::Sender<DaemonMsg>) {
    loop {
        let server = match PipeServer::create() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create pipe: {e}");
                return;
            }
        };

        let command = match server.accept_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading command: {e}");
                continue;
            }
        };

        let (reply_tx, reply_rx): (ResponseSender, _) = mpsc::channel();
        let is_stop = matches!(command, Command::Stop);

        if tx.send(DaemonMsg::Command(command, reply_tx)).is_err() {
            return;
        }

        if let Ok(response) = reply_rx.recv() {
            let _ = server.send_response(&response);
        }

        if is_stop {
            return;
        }
    }
}
