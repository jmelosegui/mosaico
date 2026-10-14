use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::*;
use crate::Error;
use crate::system::InstanceLock;

/// The process id of the daemon under test.
const PID: u32 = 4242;

/// The executable the daemon under test runs from.
const EXE: &str = "mosaico-under-test";

/// What the fakes saw happen, in order.
type Journal = Rc<RefCell<Vec<String>>>;

/// A lock that another daemon may hold already.
struct FakeLock {
    journal: Journal,
    held_elsewhere: bool,
    /// The PID file, to tell whether it is still there when the lock is
    /// released.
    pid_path: PathBuf,
}

impl SingleInstance for FakeLock {
    fn acquire(&self, _instance: &Instance) -> Result<InstanceLock> {
        if self.held_elsewhere {
            self.journal.borrow_mut().push("refused".into());
            return Err(Error::AlreadyRunning);
        }
        self.journal.borrow_mut().push("locked".into());
        let journal = Rc::clone(&self.journal);
        let pid_path = self.pid_path.clone();
        Ok(InstanceLock::new(move || {
            let pid_file = if pid_path.exists() { "with" } else { "without" };
            journal
                .borrow_mut()
                .push(format!("released {pid_file} the PID file"));
        }))
    }
}

/// An installer that only notes the leftovers it is asked to delete.
struct FakeInstaller {
    journal: Journal,
    fails: bool,
}

impl Installer for FakeInstaller {
    fn asset_name(&self) -> Result<&str> {
        Ok("mosaico.zip")
    }

    fn binary_name(&self) -> &str {
        "mosaico"
    }

    fn replace(&self, _exe: &Path, _binary: &[u8]) -> Result<()> {
        Ok(())
    }

    fn remove_leftovers(&self, exe: &Path) -> Result<()> {
        self.journal
            .borrow_mut()
            .push(format!("leftovers of {}", exe.display()));
        if self.fails {
            return Err(Error::platform("delete the old executable", "in use"));
        }
        Ok(())
    }
}

/// A daemon of an instance in a fresh temporary directory.
struct Harness {
    instance: Instance,
    journal: Journal,
    lock: FakeLock,
    installer: FakeInstaller,
    exe: Option<PathBuf>,
}

impl Harness {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir()
            .join("mosaico-lifecycle-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let instance = Instance::new(None, dir.join("config"), dir.join("data")).unwrap();
        let journal = Journal::default();
        Self {
            lock: FakeLock {
                journal: Rc::clone(&journal),
                held_elsewhere: false,
                pid_path: instance.pid_path(),
            },
            installer: FakeInstaller {
                journal: Rc::clone(&journal),
                fails: false,
            },
            exe: Some(PathBuf::from(EXE)),
            instance,
            journal,
        }
    }

    /// Runs the daemon with `serve` as its loop.
    fn run(&self, serve: &dyn Fn(&Instance) -> Result<()>) -> Result<()> {
        run(Startup {
            instance: &self.instance,
            lock: &self.lock,
            installer: &self.installer,
            exe: self.exe.clone(),
            pid: PID,
            serve,
        })
    }

    /// Notes something the loop saw.
    fn note(&self, entry: impl Into<String>) {
        self.journal.borrow_mut().push(entry.into());
    }

    fn journal(&self) -> Vec<String> {
        self.journal.borrow().clone()
    }

    fn pid_file(&self) -> PidFile {
        PidFile::new(self.instance.pid_path())
    }
}

#[test]
fn a_daemon_holds_its_lock_and_pid_file_while_it_serves_and_gives_both_back() {
    // Arrange
    let h = Harness::new("serves");
    let serve = |instance: &Instance| {
        assert_eq!(instance, &h.instance, "the loop runs the daemon's instance");
        h.note(format!(
            "serving, PID file {:?}",
            h.pid_file().read().unwrap()
        ));
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert -- the PID file is gone before the lock is released, so a
    // daemon that starts next never has its PID file removed by this one.
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        h.journal(),
        [
            format!("leftovers of {EXE}"),
            "locked".to_string(),
            format!("serving, PID file Some({PID})"),
            "released without the PID file".to_string(),
        ]
    );
    assert_eq!(h.pid_file().read().unwrap(), None);
}

#[test]
fn a_second_daemon_is_refused_and_leaves_the_running_daemons_pid_file_alone() {
    // Arrange -- another daemon holds the lock and recorded its process.
    let mut h = Harness::new("refused");
    h.lock.held_elsewhere = true;
    h.pid_file().write(77).unwrap();
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert -- leftovers are still cleared, as before the lock.
    assert!(matches!(result, Err(Error::AlreadyRunning)), "{result:?}");
    assert_eq!(
        h.journal(),
        [format!("leftovers of {EXE}"), "refused".into()]
    );
    assert_eq!(h.pid_file().read().unwrap(), Some(77));
}

#[test]
fn a_loop_that_fails_still_removes_the_pid_file_and_releases_the_lock() {
    // Arrange
    let h = Harness::new("loop-fails");
    let serve = |_: &Instance| Err(Error::platform("start the event loop", "no desktop"));

    // Act
    let result = h.run(&serve);

    // Assert -- the loop's own error, for the daemon command to print.
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "start the event loop",
                ..
            })
        ),
        "{result:?}"
    );
    assert_eq!(
        h.journal().last().map(String::as_str),
        Some("released without the PID file")
    );
    assert_eq!(h.pid_file().read().unwrap(), None);
}

#[test]
fn a_pid_file_that_cannot_be_written_stops_the_daemon_before_its_loop() {
    // Arrange -- a file where the data directory should be.
    let h = Harness::new("pid-unwritable");
    let data_dir = h.instance.data_dir();
    fs::create_dir_all(data_dir.parent().unwrap()).unwrap();
    fs::write(data_dir, "not a directory").unwrap();
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(matches!(result, Err(Error::Io(_))), "{result:?}");
    assert_eq!(
        h.journal(),
        [
            format!("leftovers of {EXE}"),
            "locked".into(),
            "released without the PID file".into(),
        ]
    );
}

#[test]
fn leftovers_that_cannot_be_deleted_do_not_stop_the_daemon() {
    // Arrange
    let mut h = Harness::new("leftovers-stay");
    h.installer.fails = true;
    let serve = |_: &Instance| {
        h.note("serving");
        Ok(())
    };

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(
        h.journal().contains(&"serving".to_string()),
        "{:?}",
        h.journal()
    );
}

#[test]
fn no_leftovers_are_looked_for_when_the_executable_is_unknown() {
    // Arrange
    let mut h = Harness::new("no-exe");
    h.exe = None;
    let serve = |_: &Instance| Ok(());

    // Act
    let result = h.run(&serve);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        h.journal(),
        ["locked", "released without the PID file"].map(String::from)
    );
}
