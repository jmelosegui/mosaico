use std::sync::mpsc;

use mosaico_core::ipc::{Served, serve};

use crate::ipc::PipeServer;

use super::daemon_types::{DaemonMsg, ResponseSender};

/// Answers CLI connections on `pipe_name` and forwards each command to
/// the main daemon thread. Runs on a dedicated thread.
///
/// If the pipe can no longer accept connections the daemon has lost its
/// only control channel: `mosaico stop` could not reach it. Rather than
/// keep tiling with no way to stop it, it asks the main thread to shut
/// down, which restores every window.
pub(super) fn ipc_loop(tx: mpsc::Sender<DaemonMsg>, pipe_name: String) {
    let mut server = PipeServer::new(&pipe_name);
    let served = serve(&mut server, |command| {
        let (reply_tx, reply_rx): (ResponseSender, _) = mpsc::channel();
        tx.send(DaemonMsg::Command(command, reply_tx)).ok()?;
        reply_rx.recv().ok()
    });

    if let Served::Failed(e) = served {
        mosaico_core::log_error!("IPC listener on {pipe_name} failed, shutting down: {e}");
        let _ = tx.send(DaemonMsg::IpcFailed);
    }
}
