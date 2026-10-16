use super::*;
use crate::id::wid;

fn who(process: u32) -> WindowIdentity {
    WindowIdentity {
        process,
        class: "App".into(),
    }
}

fn hide(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Hide {
        window: wid(window),
        how,
    }
}

#[test]
fn a_hide_is_recorded_and_a_show_or_reveal_forgets_it() {
    // Arrange
    let mut record = HiddenWindows::default();

    // Act / Assert
    assert!(record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7))));
    assert!(record.applied(&hide(2, HidingBehaviour::Hide), |_| Some(who(7))));
    assert!(!record.is_empty());
    let show = Effect::Show {
        window: wid(1),
        how: HidingBehaviour::Cloak,
    };
    assert!(record.applied(&show, |_| None));
    assert!(record.applied(&Effect::Reveal { window: wid(2) }, |_| None));
    assert!(record.is_empty());
}

#[test]
fn hiding_the_same_window_again_changes_nothing() {
    // Arrange
    let mut record = HiddenWindows::default();
    record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7)));

    // Act / Assert -- nothing to write again.
    assert!(!record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7))));
}

#[test]
fn a_window_that_is_already_gone_is_not_recorded() {
    // Arrange
    let mut record = HiddenWindows::default();

    // Act
    let changed = record.applied(&hide(1, HidingBehaviour::Cloak), |_| None);

    // Assert
    assert!(!changed);
    assert!(record.is_empty());
}

#[test]
fn unrelated_effects_leave_the_record_alone() {
    // Arrange
    let mut record = HiddenWindows::default();

    // Act / Assert
    assert!(!record.applied(&Effect::Minimize { window: wid(1) }, |_| Some(who(7))));
    assert!(record.is_empty());
}

#[test]
fn forgetting_a_destroyed_window_drops_it() {
    // Arrange
    let mut record = HiddenWindows::default();
    record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7)));

    // Act / Assert
    assert!(record.forget(wid(1)));
    assert!(!record.forget(wid(1)));
    assert!(record.is_empty());
}

#[test]
fn restore_shows_each_window_the_way_it_was_hidden() {
    // Arrange
    let mut record = HiddenWindows::default();
    record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7)));
    record.applied(&hide(2, HidingBehaviour::Minimize), |_| Some(who(7)));

    // Act
    let effects = record.restore(|_| Some(who(7)));

    // Assert
    assert_eq!(
        effects,
        vec![
            Effect::Show {
                window: wid(1),
                how: HidingBehaviour::Cloak
            },
            Effect::Show {
                window: wid(2),
                how: HidingBehaviour::Minimize
            },
        ]
    );
}

#[test]
fn restore_skips_windows_that_are_gone_or_not_the_same_one() {
    // Arrange
    let mut record = HiddenWindows::default();
    record.applied(&hide(1, HidingBehaviour::Cloak), |_| Some(who(7)));
    record.applied(&hide(2, HidingBehaviour::Cloak), |_| Some(who(7)));
    record.applied(&hide(3, HidingBehaviour::Cloak), |_| Some(who(7)));

    // Act -- 1 is gone, 2 belongs to another process now, 3 is another
    // class of window.
    let effects = record.restore(|w| match w.raw() {
        2 => Some(who(8)),
        3 => Some(WindowIdentity {
            process: 7,
            class: "Other".into(),
        }),
        _ => None,
    });

    // Assert
    assert!(effects.is_empty(), "{effects:?}");
}

#[test]
fn the_record_survives_a_round_trip_through_text() {
    // Arrange
    let mut record = HiddenWindows::default();
    record.applied(&hide(0x4014E, HidingBehaviour::Hide), |_| Some(who(7)));

    // Act
    let back = HiddenWindows::from_text(&record.to_text()).unwrap();

    // Assert
    assert_eq!(back, record);
}

#[test]
fn a_record_an_earlier_daemon_wrote_still_reads() {
    // Arrange -- each way of hiding is written under its config name.
    let text = r#"{"version": 1, "hidden": [
        {"window": 1, "how": "cloak", "process": 7, "class": "App"},
        {"window": 2, "how": "hide", "process": 7, "class": "App"},
        {"window": 3, "how": "minimize", "process": 7, "class": "App"}
    ]}"#;

    // Act
    let effects = HiddenWindows::from_text(text)
        .unwrap()
        .restore(|_| Some(who(7)));

    // Assert
    let shown = |window: u64, how: HidingBehaviour| Effect::Show {
        window: wid(window),
        how,
    };
    assert_eq!(
        effects,
        [
            shown(1, HidingBehaviour::Cloak),
            shown(2, HidingBehaviour::Hide),
            shown(3, HidingBehaviour::Minimize),
        ]
    );
}

#[test]
fn corrupt_or_unknown_text_is_an_error() {
    // Act / Assert
    assert!(HiddenWindows::from_text("not json").is_err());
    assert!(HiddenWindows::from_text(r#"{"version":99,"hidden":[]}"#).is_err());
}
