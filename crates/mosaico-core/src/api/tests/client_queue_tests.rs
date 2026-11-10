//! The client's queue: order, waiting, and how either end going away
//! shows at the other.

use std::thread;
use std::time::Duration;

use super::{Received, queue};

fn item(received: Received<u32>) -> Option<u32> {
    match received {
        Received::Item(item) => Some(item),
        _ => None,
    }
}

#[test]
fn items_come_oldest_first_then_the_queue_closes_when_the_senders_go() {
    // Arrange
    let (sender, receiver) = queue();
    let other = sender.clone();
    sender.send(1).unwrap();
    other.send(2).unwrap();
    drop(sender);
    drop(other);

    // Act
    let first = item(receiver.recv_timeout(Duration::ZERO));
    let second = item(receiver.recv_timeout(Duration::ZERO));
    let then = receiver.recv_timeout(Duration::MAX);

    // Assert
    assert_eq!((first, second), (Some(1), Some(2)));
    assert!(matches!(then, Received::Closed));
}

#[test]
fn a_wait_ends_when_nothing_comes_in_time_or_an_item_comes_from_another_thread() {
    // Arrange
    let (sender, receiver) = queue();

    // Act
    let quiet = receiver.recv_timeout(Duration::from_millis(20));
    let feeder = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        sender.send(7).unwrap();
        sender
    });
    let fed = item(receiver.recv_timeout(Duration::MAX));
    let _sender = feeder.join().unwrap();

    // Assert
    assert!(matches!(quiet, Received::Timeout));
    assert_eq!(fed, Some(7));
}

#[test]
fn a_send_hands_the_item_back_once_the_receiver_is_gone() {
    // Arrange
    let (sender, receiver) = queue();
    drop(receiver);

    // Act
    let sent = sender.send(3);

    // Assert
    assert_eq!(sent, Err(3));
}
