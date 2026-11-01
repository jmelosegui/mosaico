//! Running commands against the fake [`World`], and what they print.

use mosaico_core::config::ConfigStore;
use mosaico_core::pid::PidFile;

use crate::commands::context::{Context, Exit};
use crate::commands::fake_desktop::FakeDesktop;

use super::World;

/// Runs commands against a [`World`] and a [`FakeDesktop`], capturing
/// what they print.
pub struct Harness {
    pub world: World,
    pub desktop: FakeDesktop,
    pub pid_file: PidFile,
    pub config: ConfigStore,
    out: Vec<u8>,
    err: Vec<u8>,
}

impl Harness {
    /// A harness whose PID file and config files live in a fresh
    /// temporary directory named after `test`.
    pub fn new(test: &str) -> Self {
        let dir = std::env::temp_dir()
            .join("mosaico-command-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self {
            world: World::default(),
            desktop: FakeDesktop::default(),
            pid_file: PidFile::new(dir.join("mosaico.pid")),
            config: ConfigStore::new(dir.join("config")),
            out: Vec::new(),
            err: Vec::new(),
        }
    }

    /// Runs `command` and returns how it ended.
    pub fn run(&mut self, command: impl FnOnce(&mut Context) -> Exit) -> Exit {
        let mut cx = Context {
            daemon: &self.world,
            processes: &self.world,
            autostart: &self.world,
            windows: &self.world,
            desktop: &self.desktop,
            tiler_desktop: &|| &self.world,
            current_exe: &|| self.world.current_exe(),
            files: &self.world,
            pid_file: self.pid_file.clone(),
            config: self.config.clone(),
            clock: &self.world,
            http: &self.world.releases,
            installer: &self.world.installer,
            logo: &|_, version| self.world.logos.borrow_mut().push(version.to_owned()),
            out: &mut self.out,
            err: &mut self.err,
        };
        command(&mut cx)
    }

    /// Everything printed to standard output so far.
    pub fn out(&self) -> String {
        String::from_utf8_lossy(&self.out).into_owned()
    }

    /// Everything printed to standard error so far.
    pub fn err(&self) -> String {
        String::from_utf8_lossy(&self.err).into_owned()
    }
}
