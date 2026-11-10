use super::*;

fn key(stable: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(stable.to_string()),
        ..Default::default()
    }
}

#[test]
fn the_same_key_always_gets_the_same_id() {
    // Arrange
    let dell = key("DEL-1");

    // Act
    let first = assign_stable_ids(&[&dell], &[None]);
    let again = assign_stable_ids(&[&dell], &[None]);

    // Assert
    assert_eq!(first, again);
    assert!(first[0].as_str().starts_with("m-"));
    assert_eq!(first[0].as_str().len(), 8);
}

#[test]
fn different_keys_get_different_ids() {
    // Arrange
    let (a, b) = (key("A"), key("B"));

    // Act
    let ids = assign_stable_ids(&[&a, &b], &[None, None]);

    // Assert
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn monitors_with_the_same_key_are_told_apart_in_order() {
    // Arrange
    let unknown = MonitorKey::default();

    // Act
    let ids = assign_stable_ids(&[&unknown, &unknown, &unknown], &[None, None, None]);

    // Assert
    assert_eq!(ids[1].as_str(), format!("{}-2", ids[0]));
    assert_eq!(ids[2].as_str(), format!("{}-3", ids[0]));
}

#[test]
fn an_inherited_id_is_kept_even_when_the_key_would_give_another() {
    // Arrange
    let moved = key("on-another-port");
    let kept = StableMonitorId::new("m-41c6e2");

    // Act
    let ids = assign_stable_ids(&[&moved], &[Some(kept.clone())]);

    // Assert
    assert_eq!(ids, vec![kept]);
}

#[test]
fn a_worked_out_id_never_takes_one_a_later_monitor_inherits() {
    // Arrange: the second monitor inherits the id the first one's key
    // would give.
    let (fresh, back) = (key("A"), key("B"));
    let taken = assign_stable_ids(&[&fresh], &[None]).remove(0);

    // Act
    let ids = assign_stable_ids(&[&fresh, &back], &[None, Some(taken.clone())]);

    // Assert
    assert_eq!(ids[1], taken);
    assert_eq!(ids[0].as_str(), format!("{taken}-2"));
}

#[test]
fn an_inherited_id_held_already_is_worked_out_again() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let shared = StableMonitorId::new("m-000001");

    // Act
    let ids = assign_stable_ids(&[&a, &b], &[Some(shared.clone()), Some(shared.clone())]);

    // Assert
    assert_eq!(ids[0], shared);
    assert_ne!(ids[1], shared);
}

#[test]
fn it_is_sent_as_plain_text() {
    // Arrange
    let id = StableMonitorId::new("m-41c6e2");

    // Act
    let json = serde_json::to_string(&id).unwrap();

    // Assert
    assert_eq!(json, "\"m-41c6e2\"");
}
