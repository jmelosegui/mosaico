//! A bar mosaico knows by name, which only that bar's sessions speak for.

use std::time::Instant;

use crate::api::{ErrorCode, MonitorTarget, Peer, Reserve};
use crate::bar::BarHost;
use crate::config::BarChoice;

use super::TilingManager;
use super::bar_host_tests::{Recorder, host_on, picked};
use super::fake_platform::FakePlatform;
use super::manager_support::{start, still};

/// Session `session` of a bar that named itself `bar` in `hello`.
fn session(session: u64, bar: Option<&str>) -> Peer {
    Peer {
        session,
        client: None,
        bar: bar.map(str::to_string),
        process: Some(4000 + session as u32),
    }
}

fn top(top: u32) -> Reserve {
    Reserve {
        monitor: MonitorTarget::All,
        top,
        ..Reserve::default()
    }
}

fn on(choice: BarChoice) -> (FakePlatform, TilingManager<FakePlatform>, BarHost) {
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), choice);
    bars.start(&mut manager);
    (platform, manager, bars)
}

fn work_top(manager: &TilingManager<FakePlatform>) -> i32 {
    manager.tiler().monitors()[0].work_area().y
}

#[test]
fn yasb_lets_its_own_sessions_reserve_whatever_the_case_of_the_name() {
    // Arrange
    let (_platform, mut manager, mut bars) = on(BarChoice::Yasb);

    // Act
    let reserved = bars.reserve(&mut manager, &session(1, Some("YASB")), &top(40));

    // Assert
    assert_eq!(reserved, Ok(()));
    assert_eq!(work_top(&manager), 40);
    assert_eq!(bars.status().provider, "yasb");
}

#[test]
fn yasb_refuses_another_bar_and_a_session_that_names_none() {
    // Arrange
    let (_platform, mut manager, mut bars) = on(BarChoice::Yasb);

    // Act
    let other = bars.reserve(&mut manager, &session(1, Some("zebar")), &top(40));
    let unnamed = bars.reserve(&mut manager, &session(2, None), &top(30));

    // Assert
    let message = other.as_ref().map_err(|e| e.message.clone()).err();
    let codes = (other.map_err(|e| e.code), unnamed.map_err(|e| e.code));
    assert_eq!(
        codes,
        (Err(ErrorCode::NotProvider), Err(ErrorCode::NotProvider))
    );
    assert!(message.unwrap_or_default().contains("`external`"));
    assert_eq!(work_top(&manager), 0);
}

#[test]
fn picking_yasb_lets_go_of_the_space_other_bars_had_and_keeps_its_own() {
    // Arrange
    let (_platform, mut manager, mut bars) = on(BarChoice::External);
    let _ = bars.reserve(&mut manager, &session(1, Some("zebar")), &top(60));
    let _ = bars.reserve(&mut manager, &session(2, Some("yasb")), &top(40));

    // Act
    bars.choose(&mut manager, &picked(BarChoice::Yasb), Instant::now());

    // Assert
    assert_eq!(work_top(&manager), 40);
}

#[test]
fn yasb_keeps_the_reserve_from_the_config_before_it_connects() {
    // Arrange
    let (_platform, mut manager, mut bars) = on(BarChoice::Off);
    let mut section = picked(BarChoice::Yasb);
    section.reserve.top = 32;

    // Act
    bars.choose(&mut manager, &section, Instant::now());

    // Assert
    assert_eq!(work_top(&manager), 32);
}
