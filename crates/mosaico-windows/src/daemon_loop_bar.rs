//! What the pointer does on the bars: the sink the bars report to, and
//! the actions their clicks and the wheel ask for.

use std::sync::mpsc::Sender;
use std::time::Instant;

use mosaico_core::bar::{BarHost, BarId, PointerEvent, PointerSink};

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_commands::carry_out;
use super::daemon_types::DaemonMsg;

/// Puts what the pointer does on a bar on the daemon's queue, so it runs
/// on the main loop in order with the hotkeys and commands, against the
/// state the bar shows.
pub(super) struct ChannelSink(pub(super) Sender<DaemonMsg>);

impl PointerSink for ChannelSink {
    fn send(&self, bar: BarId, event: PointerEvent) {
        // The queue only goes away as the daemon stops.
        let _ = self.0.send(DaemonMsg::BarPointer(bar, event));
    }
}

/// Carries out the action a click on a bar asks for, the way a hotkey's
/// is carried out, on the bar's monitor. Wheel turns wait for
/// [`apply_wheel`]. Pausing pauses the hotkeys only: the bar keeps working.
pub(super) fn handle_pointer(
    bar: BarId,
    event: PointerEvent,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) {
    if let Some((action, origin)) = bars.pointer(bar, event, Instant::now()) {
        let _ = carry_out(action, origin, manager, bars, inputs, hotkeys_paused);
    }
}

/// Carries out the workspace switches the wheel asks for on the bars it
/// has gone quiet over.
pub(super) fn apply_wheel(
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) {
    for (action, origin) in bars.take_due(Instant::now()) {
        let _ = carry_out(action, origin, manager, bars, inputs, hotkeys_paused);
    }
}
