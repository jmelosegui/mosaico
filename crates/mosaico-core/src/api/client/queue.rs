//! The queue the client's threads feed its loop through: many senders,
//! one receiver. It stands in for the standard channel, which brings its
//! three implementations along for every type it carries.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// A new queue.
pub(super) fn queue<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        state: Mutex::new(State {
            items: VecDeque::new(),
            senders: 1,
            receiving: true,
        }),
        changed: Condvar::new(),
    });
    (Sender(Arc::clone(&shared)), Receiver(shared))
}

struct Shared<T> {
    state: Mutex<State<T>>,
    changed: Condvar,
}

struct State<T> {
    items: VecDeque<T>,
    senders: usize,
    receiving: bool,
}

impl<T> Shared<T> {
    fn lock(&self) -> MutexGuard<'_, State<T>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Puts items on the queue.
pub(super) struct Sender<T>(Arc<Shared<T>>);

impl<T> Sender<T> {
    /// Queues `item`, or hands it back when the receiver is gone.
    pub fn send(&self, item: T) -> Result<(), T> {
        let mut state = self.0.lock();
        if !state.receiving {
            return Err(item);
        }
        state.items.push_back(item);
        self.0.changed.notify_one();
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.0.lock().senders += 1;
        Self(Arc::clone(&self.0))
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.0.lock().senders -= 1;
        self.0.changed.notify_one();
    }
}

/// What waiting on the queue gave.
pub(super) enum Received<T> {
    /// The oldest item.
    Item(T),
    /// None came in time.
    Timeout,
    /// None is left, and every sender is gone.
    Closed,
}

/// Takes items off the queue, oldest first.
pub(super) struct Receiver<T>(Arc<Shared<T>>);

impl<T> Receiver<T> {
    /// The oldest item, waiting up to `wait` for one. A wait too long to
    /// reckon a deadline for waits until an item comes or the senders go.
    pub fn recv_timeout(&self, wait: Duration) -> Received<T> {
        let deadline = Instant::now().checked_add(wait);
        let mut state = self.0.lock();
        loop {
            if let Some(item) = state.items.pop_front() {
                return Received::Item(item);
            }
            if state.senders == 0 {
                return Received::Closed;
            }
            state = match deadline {
                None => (self.0.changed.wait(state)).unwrap_or_else(PoisonError::into_inner),
                Some(deadline) => {
                    let left = deadline.saturating_duration_since(Instant::now());
                    if left.is_zero() {
                        return Received::Timeout;
                    }
                    let waited = self.0.changed.wait_timeout(state, left);
                    waited.unwrap_or_else(PoisonError::into_inner).0
                }
            };
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut state = self.0.lock();
        state.receiving = false;
        state.items.clear();
    }
}

#[cfg(test)]
#[path = "../tests/client_queue_tests.rs"]
mod tests;
