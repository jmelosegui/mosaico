//! Metric reports and the baselines they are checked against.
//!
//! A [`Report`] holds counts measured from the code or from running it,
//! such as how many changes the tiler asks the platform for when a window
//! opens. A [`Baseline`] is the report committed to the repository. A
//! check fails when a count moves either way: a worse one is a
//! regression, and a better one has to be recorded so it cannot slip
//! back unnoticed.

mod baseline;
mod compare;
mod report;

pub use baseline::{Baseline, Failure};
pub use compare::{Change, Values, compare};
pub use report::{Better, Metric, Report};
