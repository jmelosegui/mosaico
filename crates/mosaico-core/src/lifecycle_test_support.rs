//! The fakes and the harness the daemon lifecycle tests run on.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::*;
use crate::Error;
use crate::system::InstanceLock;

/// The process id of the daemon under test.
pub(super) const PID: u32 = 4242;

/// The executable the daemon under test runs from.
pub(super) const EXE: &str = "mosaico-under-test";

/// What the fakes saw happen, in order.
pub(super) type Journal = Rc<RefCell<Vec<String>>>;

/// A lock that another daemon may hold already.
pub(super) struct FakeLock {
    pub(super) journal: Journal,
    pub(super) held_elsewhere: bool,
    /// The PID file, to tell whether it is still there when the lock is
    /// released.
    pub(super) pid_path: PathBuf,
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
pub(super) struct FakeInstaller {
    pub(super) journal: Journal,
    pub(super) fails: bool,
}

impl Installer for FakeInstaller {
    fn asset_name(&self) -> Result<&str> {
        Ok("mosaico.zip")
    }

    fn binary_name(&self) -> &str {
        "mosaico"
    }

    fn settings_name(&self) -> &str {
        "mosaico-settings"
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
pub(super) struct Harness {
    pub(super) instance: Instance,
    pub(super) journal: Journal,
    pub(super) lock: FakeLock,
    pub(super) installer: FakeInstaller,
    pub(super) exe: Option<PathBuf>,
}

impl Harness {
    pub(super) fn new(test: &str) -> Self {
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
    pub(super) fn run(&self, serve: &dyn Fn(&Instance) -> Result<()>) -> Result<()> {
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
    pub(super) fn note(&self, entry: impl Into<String>) {
        self.journal.borrow_mut().push(entry.into());
    }

    pub(super) fn journal(&self) -> Vec<String> {
        self.journal.borrow().clone()
    }

    pub(super) fn pid_file(&self) -> PidFile {
        PidFile::new(self.instance.pid_path())
    }
}
