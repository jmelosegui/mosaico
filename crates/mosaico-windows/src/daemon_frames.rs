//! The display frames the daemon's main loop moves windows on.

use std::sync::mpsc::Sender;

use mosaico_core::daemon::{FrameClock, FramePacer};

use crate::frame_clock::DisplayWait;
use crate::tiling::TilingManager;

use super::daemon_types::DaemonMsg;

/// The display frames the loop asks for while windows move, if the pacer
/// that sends them could start.
pub(super) struct Frames(Option<FramePacer>);

impl FrameClock for Frames {
    fn want_frames(&self, on: bool) {
        if let Some(pacer) = &self.0 {
            pacer.want_frames(on);
        }
    }
}

/// Starts the pacer that sends the loop a `DaemonMsg::Frame` on every
/// display refresh while it wants frames. When the pacer cannot start, the
/// daemon runs without frames: a warning says transitions are off for the
/// session, and `manager` places every window at once from then on, as on a
/// platform without frames.
pub(super) fn start(tx: Sender<DaemonMsg>, manager: &mut TilingManager) -> Frames {
    let send = move |token| tx.send(DaemonMsg::Frame(token)).is_ok();
    match FramePacer::start(DisplayWait::new, send) {
        Ok(pacer) => Frames(Some(pacer)),
        Err(e) => {
            mosaico_core::log_warn!(
                "Transitions are off for this session: display frames cannot be paced ({e})"
            );
            manager.turn_off_transitions();
            Frames(None)
        }
    }
}
