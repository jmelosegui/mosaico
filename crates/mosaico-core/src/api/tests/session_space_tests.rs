//! The requests that reserve and release space, as a session hands them
//! to the daemon loop.

use serde_json::json;

use crate::StableMonitorId;
use crate::api::{LoopRequest, MonitorTarget, Peer, Release, Reserve};

use super::session_world::{HELLO, World, state};

/// The next reservation asked of the loop, with who asked.
fn reserved(world: &World) -> (Peer, Reserve) {
    loop {
        if let LoopRequest::Reserve {
            peer,
            reserve,
            reply,
        } = world.request()
        {
            reply.ok();
            return (peer, reserve);
        }
    }
}

#[test]
fn a_reservation_reaches_the_loop_with_the_bar_and_the_process_the_system_reports() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let mut bar = world.listener.connect_from(usize::MAX, Some(77));
    bar.send(r#"{"hello":{"api":[1],"bar":"yasb"}}"#);
    let _ = (bar.recv(), bar.recv());

    // Act
    bar.send(r#"{"id":1,"reserve":{"monitor":"m-1","top":32}}"#);
    let (peer, reserve) = reserved(&world);

    // Assert
    assert_eq!(
        (peer.bar.as_deref(), peer.process),
        (Some("yasb"), Some(77))
    );
    let expected = Reserve {
        monitor: MonitorTarget::One(StableMonitorId::new("m-1")),
        top: 32,
        ..Reserve::default()
    };
    assert_eq!(reserve, expected);
    let answer = bar.recv().unwrap();
    assert_eq!(
        (&answer["id"], answer.get("ok").is_some()),
        (&json!(1), true)
    );
}

#[test]
fn a_helper_names_the_bar_process_in_its_hello() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let mut bar = world.listener.connect_from(usize::MAX, Some(77));
    bar.send(r#"{"hello":{"api":[1],"process":4242}}"#);
    let _ = (bar.recv(), bar.recv());

    // Act
    bar.send(r#"{"id":1,"reserve":{"monitor":"all","top":32}}"#);
    let (peer, _) = reserved(&world);

    // Assert
    assert_eq!(peer.process, Some(4242));
}

#[test]
fn a_session_that_reserved_tells_the_loop_it_ended_and_one_that_did_not_does_not() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (reserving, _, _) = world.open(HELLO);
    reserving.send(r#"{"id":1,"reserve":{"monitor":"all","top":32}}"#);
    let (peer, _) = reserved(&world);
    let (quiet, _, _) = world.open(HELLO);

    // Act
    quiet.leave();
    let nothing = world.sessions_become(1) && world.quiet();
    reserving.leave();
    let ended = world.request();

    // Assert
    assert!(nothing, "a session that did not reserve ends on its own");
    assert!(
        matches!(ended, LoopRequest::Ended { session } if session == peer.session),
        "{ended:?}"
    );
}

#[test]
fn a_release_reaches_the_loop_for_its_session() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (bar, _, _) = world.open(HELLO);
    bar.send(r#"{"id":1,"reserve":{"monitor":"all","top":32}}"#);
    let (peer, _) = reserved(&world);

    // Act
    bar.send(r#"{"id":2,"release":{}}"#);
    let released = loop {
        if let LoopRequest::Release {
            session,
            release,
            reply,
        } = world.request()
        {
            reply.ok();
            break (session, release);
        }
    };

    // Assert
    assert_eq!(released, (peer.session, Release::default()));
}
