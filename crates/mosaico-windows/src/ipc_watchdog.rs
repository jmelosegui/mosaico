//! Gives up on a blocking pipe read after a deadline.
//!
//! Pipe handles opened without `FILE_FLAG_OVERLAPPED` have no read
//! timeout: `ReadFile` waits until the client writes or goes away. A
//! watchdog thread waits alongside the read and, if time runs out first,
//! calls `CancelSynchronousIo` on the reading thread, which makes the
//! pending `ReadFile` fail with `ERROR_OPERATION_ABORTED`. The cancel is
//! repeated until the watchdog is disarmed, because one that lands
//! before the thread has entered `ReadFile` has nothing to cancel.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::IO::CancelSynchronousIo;
use windows::Win32::System::Threading::{GetCurrentThreadId, OpenThread, THREAD_TERMINATE};

/// How often a fired watchdog repeats the cancel until it is disarmed.
const CANCEL_RETRY: Duration = Duration::from_millis(10);

/// The reading thread's handle, owned by the watchdog.
struct ThreadHandle(HANDLE);

// SAFETY: a thread handle is a kernel object reference that any thread
// of the process may use; it is only used for CancelSynchronousIo and
// closed once, on drop.
unsafe impl Send for ThreadHandle {}

impl ThreadHandle {
    /// Cancels the thread's pending synchronous I/O, if it has any.
    fn cancel_io(&self) {
        // SAFETY: the handle came from OpenThread with THREAD_TERMINATE
        // and is still open. With no pending synchronous I/O on the
        // thread the call fails harmlessly.
        unsafe {
            let _ = CancelSynchronousIo(self.0);
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        // SAFETY: the handle came from OpenThread and is closed only here.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Cancels the calling thread's blocking I/O if it is still running when
/// the timeout passes.
pub(super) struct Watchdog {
    stop: Option<mpsc::Sender<()>>,
    fired: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Watchdog {
    /// Starts watching the calling thread. Arm it right before the read
    /// it guards and [`disarm`](Self::disarm) it right after, so the
    /// cancellation can only hit that read.
    pub(super) fn arm(timeout: Duration) -> io::Result<Self> {
        // SAFETY: OpenThread on the current thread's id returns a new
        // handle owned by ThreadHandle. THREAD_TERMINATE is the access
        // CancelSynchronousIo needs.
        let handle = unsafe { OpenThread(THREAD_TERMINATE, false, GetCurrentThreadId()) }
            .map_err(io::Error::from)?;
        let handle = ThreadHandle(handle);
        let (stop, stopped) = mpsc::channel::<()>();
        let fired = Arc::new(AtomicBool::new(false));
        let fired_flag = Arc::clone(&fired);
        let thread = thread::spawn(move || {
            if let Err(mpsc::RecvTimeoutError::Timeout) = stopped.recv_timeout(timeout) {
                fired_flag.store(true, Ordering::SeqCst);
                // A cancel only reaches I/O that is already pending. If
                // the reading thread has not entered the read yet, the
                // first one is lost, so keep cancelling until disarmed.
                handle.cancel_io();
                while let Err(mpsc::RecvTimeoutError::Timeout) = stopped.recv_timeout(CANCEL_RETRY)
                {
                    handle.cancel_io();
                }
            }
        });
        Ok(Self {
            stop: Some(stop),
            fired,
            thread: Some(thread),
        })
    }

    /// Stops watching. Returns whether the timeout ran out first.
    pub(super) fn disarm(mut self) -> bool {
        self.stop_and_join();
        self.fired.load(Ordering::SeqCst)
    }

    fn stop_and_join(&mut self) {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Watchdog {
    /// A watchdog dropped without being disarmed, on an early return or
    /// a panic, still stops before the thread goes on to other I/O.
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Read, Write};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use mosaico_core::ipc::{Command, IpcClient, Response, serve_with_timeout};

    use super::super::{PipeClient, PipeServer};

    #[test]
    fn a_client_that_stops_halfway_through_its_request_is_given_up_on() {
        // Arrange -- a client sends the start of a request and stalls, so
        // the deadline has to cover the whole request, not each read.
        let name = format!(r"\\.\pipe\mosaico-test-halfway-{}", std::process::id());
        let server_name = name.clone();
        let server = thread::spawn(move || {
            let mut server = PipeServer::new(&server_name);
            serve_with_timeout(&mut server, Duration::from_millis(200), |command| {
                Some(Response::ok_with_message(format!("{command:?}")))
            })
        });
        let client = PipeClient::new(&name);
        for _ in 0..100 {
            if client.is_daemon_running() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let mut stalled = client.connect().expect("the stalled client connects");
        stalled.write_all(b"{\"version\":1,").unwrap();

        // Act
        let (reply_tx, reply_rx) = mpsc::channel();
        let next_name = name.clone();
        thread::spawn(move || {
            let _ = reply_tx.send(PipeClient::new(&next_name).send(&Command::Status));
        });

        // Assert
        let reply = reply_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("the next client was never answered")
            .expect("the next client gets a reply");
        assert_eq!(reply.message.as_deref(), Some("Status"));

        // Cleanup
        drop(stalled);
        client.send(&Command::Stop).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn a_client_trickling_its_request_cannot_extend_the_deadline() {
        // Arrange -- a client keeps sending a byte well inside the
        // deadline, but never finishes its request. A deadline per read
        // would let it hold the server for as long as it keeps trickling.
        let name = format!(r"\\.\pipe\mosaico-test-trickle-{}", std::process::id());
        let server_name = name.clone();
        let server = thread::spawn(move || {
            let mut server = PipeServer::new(&server_name);
            serve_with_timeout(&mut server, Duration::from_millis(200), |command| {
                Some(Response::ok_with_message(format!("{command:?}")))
            })
        });
        let client = PipeClient::new(&name);
        for _ in 0..100 {
            if client.is_daemon_running() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let mut trickling = client.connect().expect("the trickling client connects");
        trickling.write_all(b"{").unwrap();
        let trickler = thread::spawn(move || {
            // Up to 3 s of one byte every 50 ms, until the server hangs up.
            for _ in 0..60 {
                thread::sleep(Duration::from_millis(50));
                if trickling.write_all(b" ").is_err() {
                    break;
                }
            }
        });

        // Act
        let started = Instant::now();
        let (reply_tx, reply_rx) = mpsc::channel();
        let next_name = name.clone();
        thread::spawn(move || {
            let _ = reply_tx.send(PipeClient::new(&next_name).send(&Command::Status));
        });

        // Assert -- answered long before the trickle would end on its own.
        let reply = reply_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("the next client waited for the trickling one")
            .expect("the next client gets a reply");
        assert_eq!(reply.message.as_deref(), Some("Status"));
        assert!(started.elapsed() < Duration::from_secs(1));

        // Cleanup
        trickler.join().unwrap();
        client.send(&Command::Stop).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn a_watchdog_that_fires_before_the_read_starts_still_cancels_it() {
        // Arrange -- the deadline passes before the guarded read begins,
        // as when the reading thread is preempted between arming the
        // watchdog and entering the read. The writer stays open and
        // silent, so nothing but the watchdog can end the read.
        let (mut reader, writer) = io::pipe().unwrap();
        let (done_tx, done_rx) = mpsc::channel();

        // Act
        thread::spawn(move || {
            let watchdog = super::Watchdog::arm(Duration::from_millis(1)).unwrap();
            thread::sleep(Duration::from_millis(50));
            let mut buf = [0u8; 1];
            let result = reader.read(&mut buf);
            let fired = watchdog.disarm();
            let _ = done_tx.send((result.is_err(), fired));
        });

        // Assert
        let (failed, fired) = done_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("the read blocked: the cancel was lost because it came before the read");
        assert!(fired);
        assert!(failed, "the cancelled read must fail");

        // Cleanup -- a blocked read ends once the writer goes away.
        drop(writer);
    }

    #[test]
    fn a_watchdog_that_is_disarmed_in_time_does_not_fire() {
        // Arrange
        let watchdog = super::Watchdog::arm(Duration::from_secs(5)).unwrap();

        // Act / Assert
        assert!(!watchdog.disarm());
    }
}
