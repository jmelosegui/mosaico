use super::support::*;

#[test]
fn events_travel_as_json_named_by_their_kind() {
    // Arrange
    let event = DesktopEvent::WorkspaceChanged {
        monitor: mid(1),
        workspace: 2,
    };

    // Act
    let json = serde_json::to_value(&event).unwrap();

    // Assert
    assert_eq!(json["event"], "workspace_changed");
    assert_eq!(json["workspace"], 2);
    assert_eq!(event.kind().name(), "workspace_changed");
    let back: DesktopEvent = serde_json::from_value(json).unwrap();
    assert_eq!(back, event);
}

#[test]
fn a_unit_event_travels_with_only_its_name() {
    // Act
    let json = serde_json::to_string(&DesktopEvent::ConfigReloaded).unwrap();

    // Assert
    assert_eq!(json, r#"{"event":"config_reloaded"}"#);
}

#[test]
fn every_kind_parses_from_its_name() {
    for kind in EventKind::ALL {
        // Act / Assert
        assert_eq!(kind.name().parse::<EventKind>(), Ok(kind));
        assert_eq!(
            serde_json::to_value(kind).unwrap(),
            serde_json::Value::from(kind.name())
        );
    }
}

#[test]
fn an_unknown_name_is_refused_with_the_known_ones() {
    // Act
    let err = "focus_change".parse::<EventKind>().unwrap_err();

    // Assert
    let text = err.to_string();
    assert!(text.starts_with("unknown event `focus_change`"), "{text}");
    assert!(text.contains("focus_changed"), "{text}");
}
