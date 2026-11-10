//! The space bars reserve through the bar API, and their windows, which
//! are left untiled.

use std::sync::Arc;
use std::time::Instant;

use crate::api::session_world::{World, state};
use crate::api::{BarSession, BarStatus, ErrorCode, MonitorTarget, Peer, Reserve};
use crate::bar::{BarHost, RESERVATION_GRACE};
use crate::config::BarChoice;
use crate::id::wid;
use crate::{StableMonitorId, WindowEvent};

use super::bar_host_tests::{Recorder, host, host_on, picked};
use super::fake_platform::FakePlatform;
use super::manager_support::{start, still, tiled};
use super::{MON_H, TilingManager};

/// Session 1, a bar of process 4242.
fn peer() -> Peer {
    Peer {
        session: 1,
        client: None,
        bar: Some("yasb".to_string()),
        process: Some(4242),
    }
}

fn top(monitor: MonitorTarget, top: u32) -> Reserve {
    Reserve {
        monitor,
        top,
        ..Reserve::default()
    }
}

/// Another program's bar, running on one monitor with windows 1 and 2.
fn external() -> (FakePlatform, TilingManager<FakePlatform>, BarHost) {
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::External);
    bars.start(&mut manager);
    (platform, manager, bars)
}

fn work_top(manager: &TilingManager<FakePlatform>) -> i32 {
    manager.tiler().monitors()[0].work_area().y
}

#[test]
fn a_bar_reserving_under_another_programs_bar_keeps_the_tiles_clear_until_it_releases() {
    // Arrange
    let (_platform, mut manager, mut bars) = external();

    // Act
    let reserved = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));
    let kept = work_top(&manager);
    bars.release(&mut manager, 1, None);

    // Assert
    assert_eq!(reserved, Ok(()));
    assert_eq!((kept, work_top(&manager)), (40, 0));
}

#[test]
fn a_reservation_is_refused_while_the_built_in_bar_keeps_the_space() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    let before = work_top(&manager);

    // Act
    let refused = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));

    // Assert
    let code = refused.map_err(|e| e.code);
    assert_eq!(code, Err(ErrorCode::NotProvider));
    assert_eq!(work_top(&manager), before);
}

#[test]
fn a_reservation_on_a_monitor_not_connected_or_past_a_quarter_of_one_is_refused() {
    // Arrange
    let (_platform, mut manager, mut bars) = external();
    let elsewhere = MonitorTarget::One(StableMonitorId::new("m-gone"));
    let deep = (MON_H / 4 + 1) as u32;

    // Act
    let gone = bars.reserve(&mut manager, &peer(), &top(elsewhere, 40));
    let large = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, deep));

    // Assert
    let codes = (gone.map_err(|e| e.code), large.map_err(|e| e.code));
    assert_eq!(codes, (Err(ErrorCode::NoMonitor), Err(ErrorCode::TooLarge)));
    assert_eq!(work_top(&manager), 0);
}

#[test]
fn the_space_of_a_bar_that_left_is_kept_for_the_grace_then_let_go() {
    // Arrange
    let (_platform, mut manager, mut bars) = external();
    let _ = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));
    let now = Instant::now();

    // Act
    bars.session_ended(1, now);
    let due = bars.due();
    bars.wake(&mut manager, now + RESERVATION_GRACE / 2);
    let early = work_top(&manager);
    bars.wake(&mut manager, now + RESERVATION_GRACE);

    // Assert
    assert_eq!(due, Some(now + RESERVATION_GRACE));
    assert_eq!((early, work_top(&manager)), (40, 0));
    assert_eq!(bars.due(), None);
}

#[test]
fn picking_a_bar_that_keeps_its_own_space_lets_go_of_the_reservations() {
    // Arrange
    let (_platform, mut manager, mut bars) = external();
    let _ = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));

    // Act
    bars.choose(&mut manager, &picked(BarChoice::Off), Instant::now());

    // Assert
    assert_eq!(work_top(&manager), 0);
}

#[test]
fn stopping_lets_go_of_the_space_a_bar_that_left_still_had() {
    // Arrange
    let (_platform, mut manager, mut bars) = external();
    let _ = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));
    bars.session_ended(1, Instant::now());

    // Act
    bars.stop();

    // Assert
    assert_eq!(bars.due(), None);
}

#[test]
fn the_windows_of_a_bar_that_reserves_leave_the_layout_and_others_stay() {
    // Arrange
    let (platform, mut manager, mut bars) = external();
    platform.edit(2, |w| w.process = 4242);
    let before = tiled(&manager, 0);

    // Act
    let _ = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));

    // Assert
    assert_eq!(before.len(), 2);
    assert_eq!(tiled(&manager, 0), [1]);
}

#[test]
fn a_window_a_reserving_bar_opens_is_not_tiled() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (3, 0)]);
    platform.edit(3, |w| w.visible = false);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::External);
    bars.start(&mut manager);
    let _ = bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40));

    // Act
    platform.edit(3, |w| {
        w.visible = true;
        w.process = 4242;
    });
    manager.handle_event(&WindowEvent::Created { window: wid(3) });

    // Assert
    assert_eq!(tiled(&manager, 0), [1]);
}

#[test]
fn the_status_lists_the_open_sessions_with_their_space_then_those_that_ended() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let mut bar = world.listener.connect_from(usize::MAX, Some(4242));
    bar.send(r#"{"hello":{"api":[1],"bar":"yasb","client":"yasb 2.0.8"}}"#);
    let _ = (bar.recv(), bar.recv());
    let (_platform, mut manager, mut bars) = external();
    bars.publish_to(Arc::clone(&world.hub));
    let gone = Peer {
        session: 9,
        client: Some("gone 1".to_string()),
        bar: None,
        process: Some(7),
    };

    // Act
    bars.reserve(&mut manager, &peer(), &top(MonitorTarget::All, 40))
        .unwrap();
    bars.reserve(&mut manager, &gone, &top(MonitorTarget::All, 20))
        .unwrap();
    bars.session_ended(9, Instant::now());
    let status = bars.status();

    // Assert
    let expected = BarStatus {
        provider: "external".to_string(),
        sessions: vec![
            BarSession {
                session: 1,
                client: Some("yasb 2.0.8".to_string()),
                bar: Some("yasb".to_string()),
                process: Some(4242),
                ended: false,
                may_reserve: true,
                reserves: vec![top(MonitorTarget::All, 40)],
            },
            BarSession {
                session: 9,
                client: Some("gone 1".to_string()),
                bar: None,
                process: Some(7),
                ended: true,
                may_reserve: true,
                reserves: vec![top(MonitorTarget::All, 20)],
            },
        ],
    };
    assert_eq!(status, expected);
}

#[test]
fn the_status_says_which_sessions_the_built_in_bar_lets_reserve() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let mut bar = world.listener.connect();
    bar.send(r#"{"hello":{"api":[1],"bar":"yasb"}}"#);
    let _ = (bar.recv(), bar.recv());
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    bars.publish_to(Arc::clone(&world.hub));

    // Act
    let status = bars.status();

    // Assert
    let may: Vec<(u64, bool)> = (status.sessions.iter())
        .map(|s| (s.session, s.may_reserve))
        .collect();
    assert_eq!(may, [(1, false)]);
}
