use super::*;

fn edid(manufacturer: &str, product: u16, serial: Option<u32>) -> Option<EdidIdentity> {
    Some(EdidIdentity {
        manufacturer: manufacturer.into(),
        product,
        serial,
    })
}

fn key(stable_id: Option<&str>, edid: Option<EdidIdentity>, connector: Option<&str>) -> MonitorKey {
    MonitorKey {
        stable_id: stable_id.map(Into::into),
        edid,
        connector: connector.map(Into::into),
    }
}

#[test]
fn matches_by_stable_id_regardless_of_order() {
    // Arrange
    let a = key(Some("A"), None, None);
    let b = key(Some("B"), None, None);

    // Act
    let result = match_monitors(&[a.clone(), b.clone()], &[b, a]);

    // Assert
    assert_eq!(result, vec![Some(1), Some(0)]);
}

#[test]
fn matches_by_edid_serial_when_stable_id_changes() {
    // Arrange -- same unit moved to another port.
    let old = key(Some("port1"), edid("DEL", 0x41B6, Some(7)), Some("dp:0"));
    let new = key(Some("port2"), edid("DEL", 0x41B6, Some(7)), Some("dp:1"));

    // Act / Assert
    assert_eq!(match_monitors(&[old], &[new]), vec![Some(0)]);
}

#[test]
fn different_serials_never_match() {
    // Arrange
    let old = key(None, edid("DEL", 0x41B6, Some(7)), Some("dp:0"));
    let new = key(None, edid("DEL", 0x41B6, Some(8)), Some("dp:0"));

    // Act / Assert
    assert_eq!(match_monitors(&[old], &[new]), vec![None]);
}

#[test]
fn identical_models_without_serial_are_told_apart_by_connector() {
    // Arrange -- two of the same monitor, no serial in the EDID.
    let left = key(None, edid("DEL", 0x41B6, None), Some("dp:0"));
    let right = key(None, edid("DEL", 0x41B6, None), Some("dp:1"));

    // Act
    let result = match_monitors(&[left.clone(), right.clone()], &[right, left]);

    // Assert
    assert_eq!(result, vec![Some(1), Some(0)]);
}

#[test]
fn unique_model_matches_even_on_another_port() {
    // Arrange
    let old = key(None, edid("GSM", 0x5B7F, None), Some("hdmi:0"));
    let new = key(None, edid("GSM", 0x5B7F, None), Some("dp:2"));

    // Act / Assert
    assert_eq!(match_monitors(&[old], &[new]), vec![Some(0)]);
}

#[test]
fn ambiguous_models_are_left_unmatched() {
    // Arrange -- two identical monitors, both moved to unknown ports.
    let old = vec![
        key(None, edid("DEL", 1, None), Some("dp:0")),
        key(None, edid("DEL", 1, None), Some("dp:1")),
    ];
    let new = vec![
        key(None, edid("DEL", 1, None), Some("usb:0")),
        key(None, edid("DEL", 1, None), Some("usb:1")),
    ];

    // Act / Assert
    assert_eq!(match_monitors(&old, &new), vec![None, None]);
}

#[test]
fn stronger_tier_claims_first() {
    // Arrange -- `new[0]` only matches old[1] by unique model, but
    // `new[1]` matches old[1] by stable id. The stable id must win.
    let old = vec![
        key(None, edid("AAA", 1, None), None),
        key(Some("X"), edid("BBB", 2, None), None),
    ];
    let new = vec![
        key(None, edid("BBB", 2, None), None),
        key(Some("X"), edid("BBB", 2, None), None),
    ];

    // Act
    let result = match_monitors(&old, &new);

    // Assert
    assert_eq!(result[1], Some(1));
    assert_eq!(result[0], None);
}

#[test]
fn each_old_monitor_is_used_once() {
    // Arrange
    let old = vec![key(Some("A"), None, None)];
    let new = vec![key(Some("A"), None, None), key(Some("A"), None, None)];

    // Act / Assert
    assert_eq!(match_monitors(&old, &new), vec![Some(0), None]);
}

#[test]
fn unknown_keys_never_match() {
    // Arrange
    let unknown = MonitorKey::default();

    // Act / Assert
    assert!(unknown.is_unknown());
    assert_eq!(
        match_monitors(
            std::slice::from_ref(&unknown),
            std::slice::from_ref(&unknown)
        ),
        vec![None]
    );
}

#[test]
fn removed_and_added_monitors_are_unmatched() {
    // Arrange -- laptop stays, home monitor leaves, office monitor arrives.
    let laptop = key(
        Some("internal"),
        edid("BOE", 0x0AF1, None),
        Some("internal:0"),
    );
    let home = key(Some("home"), edid("DEL", 0x41B6, Some(1)), Some("dp:0"));
    let office = key(Some("office"), edid("HPN", 0x3604, Some(2)), Some("dp:0"));

    // Act
    let result = match_monitors(&[laptop.clone(), home], &[laptop, office]);

    // Assert
    assert_eq!(result, vec![Some(0), None]);
}

#[test]
fn display_is_readable() {
    // Arrange
    let k = key(Some("path"), edid("DEL", 0x41B6, Some(7)), Some("dp:1"));

    // Act / Assert
    assert_eq!(k.to_string(), "DEL41B6 #7 on dp:1 [path]");
    assert_eq!(MonitorKey::default().to_string(), "unknown");
}

#[test]
fn key_round_trips_through_toml() {
    // Arrange
    let k = key(Some("path"), edid("DEL", 0x41B6, None), None);

    // Act
    let text = toml::to_string(&k).unwrap();
    let back: MonitorKey = toml::from_str(&text).unwrap();

    // Assert
    assert_eq!(back, k);
}
