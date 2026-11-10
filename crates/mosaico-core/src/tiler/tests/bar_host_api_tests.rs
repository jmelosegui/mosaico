//! The host publishing the state to the bar API's sessions, beside the
//! provider it runs.

use std::sync::Arc;

use crate::api::LoopRequest;
use crate::api::session_world::{HELLO, World};
use crate::config::BarChoice;

use super::bar_host_tests::{Recorder, host, host_on};
use super::fake_platform::FakePlatform;
use super::manager_support::{start, still};

#[test]
fn nothing_is_published_while_no_session_follows_the_state() {
    // Arrange
    let world = World::new();
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::External);
    bars.publish_to(Arc::clone(&world.hub));

    // Act
    bars.start(&mut manager);
    bars.set_paused(true);
    bars.show(&mut manager);

    // Assert
    assert_eq!((world.hub.rev(), world.hub.fresh()), (0, false));
}

#[test]
fn a_bar_reading_the_api_is_served_the_state_while_another_provider_runs() {
    // Arrange
    let world = World::new();
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::External);
    bars.publish_to(Arc::clone(&world.hub));
    bars.start(&mut manager);
    let mut bar = world.listener.connect();
    bar.send(HELLO);
    assert!(matches!(world.request(), LoopRequest::Publish));

    // Act -- the snapshot is read before pausing, since a writer that
    // wakes after both shows sends the latest state as one snapshot.
    bars.show(&mut manager);
    let (_, snapshot) = (bar.recv().unwrap(), bar.recv().unwrap());
    bars.set_paused(true);
    bars.show(&mut manager);
    let update = bar.recv().unwrap();

    // Assert
    assert_eq!(snapshot["snapshot"]["daemon"]["provider"], "external");
    assert_eq!(update["update"]["daemon"]["paused"], true);
}

#[test]
fn a_session_joining_after_the_state_was_shown_is_published_it_unchanged() {
    // Arrange -- the built-in bar was shown the state before any session.
    let world = World::new();
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let recorder = Recorder::default();
    let mut bars = host(&recorder);
    bars.publish_to(Arc::clone(&world.hub));
    bars.start(&mut manager);
    recorder.take();
    let mut bar = world.listener.connect();
    bar.send(HELLO);
    assert!(matches!(world.request(), LoopRequest::Publish));

    // Act
    bars.show(&mut manager);

    // Assert -- the session gets it, and the provider is not shown it again.
    let (_, snapshot) = (bar.recv().unwrap(), bar.recv().unwrap());
    assert_eq!(snapshot["snapshot"]["daemon"]["provider"], "recorder");
    assert!(recorder.shown().is_empty());
}
