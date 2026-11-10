//! Picking another bar while the daemon runs, on four windows: what the
//! tiler asks of the platform as the space a bar keeps changes, and that a
//! switch between two bars keeping the same space moves no window.

use std::time::Instant;

use crate::config::BarChoice;

use super::super::bar_host_tests::{Recorder, host_on, picked};
use super::super::metrics_support::{Measured, four, on};

/// Four windows under the bar `from`, started, then switched to `to`.
fn switch(from: BarChoice, to: BarChoice) -> Measured {
    let (platform, mut manager) = four();
    let mut bars = host_on(&Recorder::default(), from);
    bars.start(&mut manager);
    manager.flush_pending();
    on((platform, manager), |m| {
        bars.choose(m, &picked(to), Instant::now());
    })
}

/// The built-in bar given up for none: the windows take its space.
pub(super) fn to_none() -> Measured {
    switch(BarChoice::Mosaico, BarChoice::Off)
}

/// No bar, then the built-in one: the windows make room for it.
pub(super) fn to_builtin() -> Measured {
    switch(BarChoice::Off, BarChoice::Mosaico)
}

/// Another program's bar given up for none, which keep the same space.
pub(super) fn same_space() -> Measured {
    switch(BarChoice::External, BarChoice::Off)
}

/// The built-in bar given up for another program's, which holds its space
/// until that bar says how far it reaches: no window moves yet.
pub(super) fn to_external() -> Measured {
    switch(BarChoice::Mosaico, BarChoice::External)
}
