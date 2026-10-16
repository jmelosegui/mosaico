use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Change, Report, Values, compare};

/// A report committed to the repository as a TOML file, that later
/// reports must match.
#[derive(Debug, Clone, Copy)]
pub struct Baseline<'a> {
    path: &'a Path,
    bless_with: &'a str,
}

/// Why a report does not pass its baseline.
#[derive(Debug)]
pub enum Failure {
    /// There is no baseline yet.
    Missing { path: PathBuf, bless_with: String },
    /// The baseline could not be read, written or parsed.
    Unusable { path: PathBuf, reason: String },
    /// Counts differ from the baseline.
    Changed {
        path: PathBuf,
        changes: Vec<Change>,
        bless_with: String,
    },
}

impl<'a> Baseline<'a> {
    /// The baseline kept at `path`. `bless_with` is the command that
    /// records a new one, for failure messages to suggest.
    pub fn new(path: &'a Path, bless_with: &'a str) -> Self {
        Self { path, bless_with }
    }

    /// Compares `report` with the baseline and returns what changed.
    ///
    /// Any change fails the check unless `bless` is set, in which case
    /// the report replaces the baseline instead. A baseline that already
    /// matches is left untouched.
    pub fn check(&self, report: &Report, bless: bool) -> Result<Vec<Change>, Failure> {
        let current = match fs::read_to_string(self.path) {
            Ok(text) => Some(self.parse(&text)?),
            Err(_) if bless => None,
            Err(_) => return Err(self.missing()),
        };
        let changes = compare(current.as_ref().unwrap_or(&Values::new()), report);
        if bless {
            if current.is_none() || !changes.is_empty() {
                self.write(report)?;
            }
            return Ok(changes);
        }
        if changes.is_empty() {
            Ok(changes)
        } else {
            Err(Failure::Changed {
                path: self.path.to_owned(),
                changes,
                bless_with: self.bless_with.to_owned(),
            })
        }
    }

    /// The counts the baseline holds.
    pub fn load(&self) -> Result<Values, Failure> {
        match fs::read_to_string(self.path) {
            Ok(text) => self.parse(&text),
            Err(_) => Err(self.missing()),
        }
    }

    fn parse(&self, text: &str) -> Result<Values, Failure> {
        toml::from_str(text).map_err(|e| self.unusable(e))
    }

    fn write(&self, report: &Report) -> Result<(), Failure> {
        let body = toml::to_string(&report.values()).map_err(|e| self.unusable(e))?;
        let text = format!(
            "# Measured by `{}`. A change here comes from a new measurement.\n\n{body}",
            self.bless_with
        );
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| self.unusable(e))?;
        }
        fs::write(self.path, text).map_err(|e| self.unusable(e))
    }

    fn missing(&self) -> Failure {
        Failure::Missing {
            path: self.path.to_owned(),
            bless_with: self.bless_with.to_owned(),
        }
    }

    fn unusable(&self, reason: impl fmt::Display) -> Failure {
        Failure::Unusable {
            path: self.path.to_owned(),
            reason: reason.to_string(),
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { path, bless_with } => write!(
                f,
                "no metrics baseline at {}; record one with `{bless_with}`",
                path.display()
            ),
            Self::Unusable { path, reason } => {
                write!(f, "metrics baseline {}: {reason}", path.display())
            }
            Self::Changed {
                path,
                changes,
                bless_with,
            } => {
                writeln!(f, "metrics differ from {}:", path.display())?;
                for change in changes {
                    writeln!(f, "  {change}")?;
                }
                write!(
                    f,
                    "Fix what got worse. Once every change is intended, record them \
                     with `{bless_with}` and commit the baseline."
                )
            }
        }
    }
}

impl std::error::Error for Failure {}

#[cfg(test)]
#[path = "baseline_tests.rs"]
mod tests;
