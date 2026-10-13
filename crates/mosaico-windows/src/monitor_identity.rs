//! Stable monitor identity on Windows.
//!
//! `HMONITOR` values are handed out again every time the display
//! configuration changes, even for monitors that never went away, so they
//! cannot be used to recognise a monitor after a reconnect. This module
//! reads the identity the display stack keeps per physical monitor and
//! turns it into a [`MonitorKey`].

use std::collections::HashMap;

use mosaico_core::{EdidIdentity, MonitorKey, pnp_manufacturer};
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
    GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::ERROR_SUCCESS;

use crate::monitor_edid::{output_technology_name, registry_edid, wide_to_string};

/// `edidIdsValid` bit of `DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS`.
const EDID_IDS_VALID: u32 = 0x4;

/// Returns the identity of every active display, keyed by the GDI device
/// name of the source it shows (`\\.\DISPLAY1`), which is what
/// `GetMonitorInfoW` reports for the matching `HMONITOR`.
///
/// Displays that cannot be queried are simply missing from the map; the
/// caller then gets an unknown key for them.
pub(crate) fn display_identities() -> HashMap<String, MonitorKey> {
    let mut keys = HashMap::new();
    for path in active_paths() {
        let Some(gdi_name) = source_gdi_name(&path) else {
            continue;
        };
        // In clone mode several targets show the same source. The first
        // one is as good as any for telling the monitor apart.
        if keys.contains_key(&gdi_name) {
            continue;
        }
        if let Some(key) = target_key(&path) {
            keys.insert(gdi_name, key);
        }
    }
    keys
}

fn active_paths() -> Vec<DISPLAYCONFIG_PATH_INFO> {
    // The configuration can change between sizing the buffers and
    // reading them, in which case the call fails with
    // ERROR_INSUFFICIENT_BUFFER. A couple of retries covers it.
    for _ in 0..3 {
        let mut path_count = 0u32;
        let mut mode_count = 0u32;
        // SAFETY: Writes two counts into caller-owned integers.
        let sized = unsafe {
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
        };
        if sized != ERROR_SUCCESS {
            return Vec::new();
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        // SAFETY: Both buffers hold as many elements as the counts say,
        // and the call writes back how many it filled.
        let queried = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };
        if queried == ERROR_SUCCESS {
            paths.truncate(path_count as usize);
            return paths;
        }
    }
    Vec::new()
}

fn source_gdi_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
    let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
            adapterId: path.sourceInfo.adapterId,
            id: path.sourceInfo.id,
        },
        ..Default::default()
    };
    // SAFETY: The header describes the full struct it is embedded in.
    let status = unsafe { DisplayConfigGetDeviceInfo(&mut source.header) };
    (status == 0).then(|| wide_to_string(&source.viewGdiDeviceName))
}

fn target_key(path: &DISPLAYCONFIG_PATH_INFO) -> Option<MonitorKey> {
    let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
            adapterId: path.targetInfo.adapterId,
            id: path.targetInfo.id,
        },
        ..Default::default()
    };
    // SAFETY: The header describes the full struct it is embedded in.
    let status = unsafe { DisplayConfigGetDeviceInfo(&mut target.header) };
    if status != 0 {
        return None;
    }

    let device_path = wide_to_string(&target.monitorDevicePath);
    // SAFETY: Reading the plain integer view of a C bitfield union.
    let flags = unsafe { target.flags.Anonymous.value };
    // Prefer the full EDID Windows keeps in the registry, which carries
    // the serial. Fall back to the IDs the display stack reports, whose
    // manufacturer field comes back with its two bytes swapped.
    let edid = registry_edid(&device_path)
        .and_then(|bytes| EdidIdentity::from_edid(&bytes))
        .or_else(|| {
            (flags & EDID_IDS_VALID != 0).then(|| EdidIdentity {
                manufacturer: pnp_manufacturer(target.edidManufactureId.swap_bytes()),
                product: target.edidProductCodeId,
                serial: None,
            })
        });
    let connector = format!(
        "{}:{}",
        output_technology_name(target.outputTechnology.0),
        target.connectorInstance
    );

    Some(MonitorKey {
        stable_id: (!device_path.is_empty()).then_some(device_path),
        edid,
        connector: Some(connector),
    })
}
