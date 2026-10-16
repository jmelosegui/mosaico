//! What a placement posted behind what was posted to a window decides
//! from what is read of the window.

use super::*;

/// A window in its normal state, shown, whose application responds, with
/// no change of its show state ahead of the placement.
fn normal() -> Seen {
    Seen {
        maximized: false,
        minimized: false,
        in_view: None,
        shown: true,
        responding: true,
    }
}

#[test]
fn a_maximized_window_in_view_is_restored_first() {
    // Arrange -- shown, or to be once a show queued lands.
    let shown = Seen {
        maximized: true,
        ..normal()
    };
    let to_be_shown = Seen {
        shown: false,
        in_view: Some(true),
        ..shown
    };

    // Act / Assert
    assert_eq!(first(&shown), First::Restore);
    assert_eq!(first(&to_be_shown), First::Restore);
}

#[test]
fn a_maximized_window_out_of_view_has_its_style_cleared_and_is_never_restored() {
    // Arrange -- hidden, and to stay so once what was queued lands, which
    // a restore behind it would undo.
    let hidden = Seen {
        maximized: true,
        shown: false,
        ..normal()
    };
    let to_stay_hidden = Seen {
        in_view: Some(false),
        ..hidden
    };

    // Act / Assert
    assert_eq!(first(&hidden), First::ClearMaximized);
    assert_eq!(first(&to_stay_hidden), First::ClearMaximized);
}

#[test]
fn a_maximized_window_a_queued_hide_is_still_to_hide_is_left_maximized() {
    // Arrange -- its style cleared now could undo the hide, were its
    // application to get to the hide in between.
    let to_be_hidden = Seen {
        maximized: true,
        in_view: Some(false),
        ..normal()
    };

    // Act / Assert
    assert_eq!(first(&to_be_hidden), First::Nothing);
}

#[test]
fn a_maximized_window_out_of_view_whose_application_hangs_is_left_maximized() {
    // Arrange -- clearing its style would wait for it.
    let hung = Seen {
        maximized: true,
        shown: false,
        responding: false,
        ..normal()
    };

    // Act / Assert
    assert_eq!(first(&hung), First::Nothing);
}

#[test]
fn a_minimized_window_a_queued_show_brings_back_is_restored_again_behind_it() {
    // Arrange -- which ends the maximized state it may come back in.
    let minimized = Seen {
        minimized: true,
        in_view: Some(true),
        ..normal()
    };

    // Act / Assert
    assert_eq!(first(&minimized), First::Restore);
}

#[test]
fn a_minimized_window_to_stay_out_of_view_is_never_restored() {
    // Arrange -- to stay minimized or be hidden, which a restore would
    // undo, or with nothing ahead that shows it.
    let minimized = Seen {
        minimized: true,
        ..normal()
    };
    let seen = [Some(false), None].map(|in_view| Seen {
        in_view,
        ..minimized
    });

    // Act / Assert
    for seen in seen {
        assert_eq!(first(&seen), First::Nothing, "{seen:?}");
    }
    assert_eq!(first(&normal()), First::Nothing);
}

#[test]
fn the_kept_borders_are_used_only_for_a_window_out_of_its_normal_state() {
    // Arrange
    let kept = BorderOffset {
        left: 7,
        top: 0,
        right: 7,
        bottom: 7,
    };

    // Act
    let minimized = kept_border(false, Some(kept)).map(|border| border.left);
    let normal = kept_border(true, Some(kept)).map(|border| border.left);
    let none_kept = kept_border(false, None).map(|border| border.left);

    // Assert -- the others are read when the placement is posted.
    assert_eq!([minimized, normal, none_kept], [Some(7), None, None]);
}
