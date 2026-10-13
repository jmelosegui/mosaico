use serde::{Deserialize, Serialize};

use crate::Rect;

/// Identity of a physical display that survives reconnects.
///
/// Platforms hand out a fresh handle for a display every time it is
/// connected (`HMONITOR` on Windows, `CGDirectDisplayID` on macOS, RandR
/// output XIDs on X11), and they do so for displays that never went
/// away too. A `MonitorKey` is what lets the tiler recognise a display
/// when it comes back, so its workspaces can be restored.
///
/// # Contract for platform implementations
///
/// - The key must stay the same across unplugging and replugging,
///   switching a KVM away and back, sleep, reboots, and changes to
///   resolution, scaling or arrangement.
/// - It must never be built from per-session handles, enumeration order,
///   desktop position or resolution. Those all change.
/// - Two displays of the same model on different ports should differ in
///   at least one field. When the hardware makes that impossible, report
///   the shared EDID and let [`match_monitors`] resolve it.
/// - Leave a field `None` rather than invent a value. A missing field
///   only weakens matching; a wrong one causes false matches.
///
/// Matching rules live in [`match_monitors`], not in the platform, so
/// every platform restores layouts the same way.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorKey {
    /// The most specific stable identifier the platform offers.
    ///
    /// Opaque to the core: only compared for equality. Windows uses the
    /// monitor device interface path, macOS would use the display UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_id: Option<String>,
    /// Identity read from the display's EDID. Means the same on every OS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edid: Option<EdidIdentity>,
    /// The port the display is attached through, e.g. `"dp:1"`.
    ///
    /// Opaque to the core: only compared for equality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector: Option<String>,
}

/// Display identity decoded from EDID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdidIdentity {
    /// Three letter PNP manufacturer ID, e.g. `"DEL"`.
    pub manufacturer: String,
    /// Manufacturer product code.
    pub product: u16,
    /// Serial number, or `None` when the EDID leaves it blank (zero).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
}

impl EdidIdentity {
    /// Whether both describe the same model and nothing rules out the
    /// same unit. Two known serials that differ are different displays.
    fn same_model(&self, other: &EdidIdentity) -> bool {
        self.manufacturer == other.manufacturer
            && self.product == other.product
            && match (self.serial, other.serial) {
                (Some(a), Some(b)) => a == b,
                _ => true,
            }
    }
}

impl MonitorKey {
    /// Whether the platform could not identify the display at all.
    ///
    /// Such a key never matches anything in [`match_monitors`]; callers
    /// fall back to handle or position matching for it.
    pub fn is_unknown(&self) -> bool {
        self.stable_id.is_none() && self.edid.is_none() && self.connector.is_none()
    }
}

impl std::fmt::Display for MonitorKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_unknown() {
            return write!(f, "unknown");
        }
        let mut parts = Vec::new();
        if let Some(edid) = &self.edid {
            match edid.serial {
                Some(serial) => parts.push(format!(
                    "{}{:04X} #{}",
                    edid.manufacturer, edid.product, serial
                )),
                None => parts.push(format!("{}{:04X}", edid.manufacturer, edid.product)),
            }
        }
        if let Some(connector) = &self.connector {
            parts.push(format!("on {connector}"));
        }
        if let Some(id) = &self.stable_id {
            parts.push(format!("[{id}]"));
        }
        write!(f, "{}", parts.join(" "))
    }
}

/// A connected display, as reported by a platform.
///
/// Each platform crate implements this for its own monitor type, the
/// same way it implements [`crate::Window`].
pub trait Monitor {
    /// Stable identity of the display. See [`MonitorKey`] for the rules.
    fn key(&self) -> &MonitorKey;

    /// Usable area, excluding taskbars, docks and panels.
    fn work_area(&self) -> Rect;

    /// Whether this is the primary display.
    ///
    /// Windows from displays that disconnect are moved here.
    fn is_primary(&self) -> bool;
}

mod matching;

pub use matching::match_monitors;

#[cfg(test)]
mod tests;
