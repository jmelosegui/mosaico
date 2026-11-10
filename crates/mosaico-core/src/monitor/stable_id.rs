//! Ids for monitors that mean the same display across display changes,
//! for programs outside mosaico to name one by.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{EdidIdentity, MonitorKey};

/// Names a monitor for programs outside mosaico, such as another bar,
/// the same way across display changes and while mosaico runs.
///
/// The tiler gives each monitor one when it first sees it. A monitor
/// that stays connected through a display change keeps its id, and so
/// does one that disconnects and comes back, even on another port: the
/// id is parked and restored with its workspaces. Two connected monitors
/// never share one.
///
/// The text is opaque: compare it, never read meaning into it. It is
/// worked out from what the display says of itself, so the same display
/// usually gets the same id on the next start too, but nothing promises
/// that.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableMonitorId(String);

impl StableMonitorId {
    /// Wraps an id read back from where it was sent, such as a request
    /// from a bar.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The id as it is sent.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id worked out from `key` alone, before it is made unique among
    /// the connected monitors: `m-` and six hex digits of a hash of what
    /// the key holds, the same on every platform and every start.
    fn from_key(key: &MonitorKey) -> Self {
        let mut hash = Fnv::default();
        hash.text(key.stable_id.as_deref());
        match &key.edid {
            Some(EdidIdentity {
                manufacturer,
                product,
                serial,
            }) => {
                hash.text(Some(manufacturer.as_str()));
                hash.bytes(&product.to_le_bytes());
                hash.bytes(&serial.unwrap_or(0).to_le_bytes());
            }
            None => hash.bytes(&[0]),
        }
        hash.text(key.connector.as_deref());
        Self(format!("m-{:06x}", hash.0 & 0xFF_FFFF))
    }
}

impl fmt::Display for StableMonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The ids of the monitors with `keys`, in order: the one each inherits,
/// where it has one, from the monitor it continues or from when it was
/// connected before, and otherwise one worked out from its key.
///
/// No two of them are the same. An inherited id that an earlier monitor
/// holds already is worked out again, and one worked out from a key that
/// another monitor holds gets `-2`, `-3` and so on, in order, so the
/// same monitors in the same order always get the same ids.
pub fn assign_stable_ids(
    keys: &[&MonitorKey],
    inherited: &[Option<StableMonitorId>],
) -> Vec<StableMonitorId> {
    let mut kept: Vec<Option<StableMonitorId>> = Vec::with_capacity(keys.len());
    let mut taken = HashSet::new();
    for i in 0..keys.len() {
        let own = inherited.get(i).cloned().flatten();
        kept.push(own.filter(|id| taken.insert(id.clone())));
    }
    keys.iter()
        .zip(kept)
        .map(|(key, kept)| {
            kept.unwrap_or_else(|| {
                let base = StableMonitorId::from_key(key);
                let mut id = base.clone();
                let mut n = 2;
                while !taken.insert(id.clone()) {
                    id = StableMonitorId(format!("{base}-{n}"));
                    n += 1;
                }
                id
            })
        })
        .collect()
}

/// 64-bit FNV-1a, which gives the same hash on every platform and
/// release, unlike the standard library's hasher.
struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    fn bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }

    /// Feeds an optional field, so a field that is missing and one that
    /// is empty hash apart, and so do fields that run into each other.
    fn text(&mut self, text: Option<&str>) {
        match text {
            Some(text) => {
                self.bytes(&[1]);
                self.bytes(&(text.len() as u64).to_le_bytes());
                self.bytes(text.as_bytes());
            }
            None => self.bytes(&[0]),
        }
    }
}

#[cfg(test)]
#[path = "stable_id_tests.rs"]
mod tests;
