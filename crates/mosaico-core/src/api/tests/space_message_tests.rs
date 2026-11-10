//! How a bar's `reserve` and `release` requests are read.

use serde_json::json;

use crate::StableMonitorId;
use crate::api::{ClientMessage, ErrorCode, MonitorTarget, Release, Request};

#[test]
fn a_reservation_reads_its_monitor_and_leaves_unnamed_edges_at_zero() {
    // Act
    let all = ClientMessage::parse(json!({"id": 1, "reserve": {"monitor": "all", "left": 40}}));
    let one = ClientMessage::parse(json!({"id": 2, "reserve": {"monitor": "m-41c6e2"}}));

    // Assert
    let reserve = |message| match message {
        Ok(ClientMessage::Request {
            request: Request::Reserve(reserve),
            ..
        }) => reserve,
        other => panic!("not a reservation: {other:?}"),
    };
    let all = reserve(all);
    assert_eq!(
        (all.monitor, all.top, all.left),
        (MonitorTarget::All, 0, 40)
    );
    let one = reserve(one);
    assert_eq!(
        one.monitor,
        MonitorTarget::One(StableMonitorId::new("m-41c6e2"))
    );
    assert_eq!((one.top, one.bottom, one.left, one.right), (0, 0, 0, 0));
}

#[test]
fn a_reservation_without_a_monitor_is_malformed() {
    // Act
    let refused = ClientMessage::parse(json!({"id": 3, "reserve": {"top": 32}}));

    // Assert
    let refusal = refused.unwrap_err();
    assert_eq!(
        (refusal.id, refusal.error.code),
        (Some(3), ErrorCode::Malformed)
    );
}

#[test]
fn a_release_without_a_monitor_releases_all() {
    // Act
    let release = ClientMessage::parse(json!({"id": 4, "release": {}}));

    // Assert
    assert!(matches!(
        release,
        Ok(ClientMessage::Request {
            request: Request::Release(Release { monitor: None }),
            ..
        })
    ));
}
