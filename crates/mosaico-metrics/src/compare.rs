use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{Better, Report};

/// Counts by section and metric name, as a baseline keeps them.
pub type Values = BTreeMap<String, BTreeMap<String, u64>>;

/// A count that differs from its baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The section of the metric.
    pub section: String,
    /// The metric.
    pub metric: String,
    /// The count in the baseline.
    pub from: u64,
    /// The count measured now.
    pub to: u64,
    /// Whether the count moved the wrong way.
    pub worse: bool,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verdict = if self.worse { "worse " } else { "better" };
        write!(
            f,
            "{verdict} {}.{}: {} -> {}",
            self.section, self.metric, self.from, self.to
        )
    }
}

/// Every count of `report` that differs from `baseline`, in section and
/// metric order.
///
/// A metric either side leaves out counts as 0, so a report can leave out
/// work that did not happen. A metric only the baseline has is taken to
/// be one where fewer is better, as counts of work are.
pub fn compare(baseline: &Values, report: &Report) -> Vec<Change> {
    let measured = report.values();
    let sections: BTreeSet<&String> = baseline.keys().chain(measured.keys()).collect();
    let mut changes = Vec::new();
    for section in sections {
        let empty = BTreeMap::new();
        let before = baseline.get(section).unwrap_or(&empty);
        let after = measured.get(section).unwrap_or(&empty);
        let names: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
        for name in names {
            let from = before.get(name).copied().unwrap_or(0);
            let to = after.get(name).copied().unwrap_or(0);
            if from == to {
                continue;
            }
            let better = report
                .get(section, name)
                .map_or(Better::Lower, |m| m.better);
            let worse = match better {
                Better::Lower => to > from,
                Better::Higher => to < from,
            };
            changes.push(Change {
                section: section.clone(),
                metric: name.clone(),
                from,
                to,
                worse,
            });
        }
    }
    changes
}

#[cfg(test)]
#[path = "compare_tests.rs"]
mod tests;
