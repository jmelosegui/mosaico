use std::collections::BTreeMap;

/// Which way a metric is meant to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Better {
    /// Fewer is better: work done, rule breaches.
    Lower,
    /// More is better: tests.
    Higher,
}

/// A measured count and which way it is meant to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metric {
    /// The count.
    pub value: u64,
    /// Which way it is meant to move.
    pub better: Better,
}

/// Metrics grouped in named sections.
///
/// Sections and metrics are kept in name order, so the baseline a report
/// writes only changes where a count does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    sections: BTreeMap<String, BTreeMap<String, Metric>>,
}

impl Report {
    /// An empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records `value` as metric `name` of `section`, replacing what was
    /// recorded there.
    pub fn set(&mut self, section: &str, name: &str, value: u64, better: Better) {
        self.sections
            .entry(section.to_owned())
            .or_default()
            .insert(name.to_owned(), Metric { value, better });
    }

    /// Metric `name` of `section`, if it was recorded.
    pub fn get(&self, section: &str, name: &str) -> Option<Metric> {
        self.sections.get(section)?.get(name).copied()
    }

    /// Every section, in name order.
    pub fn sections(&self) -> &BTreeMap<String, BTreeMap<String, Metric>> {
        &self.sections
    }

    /// The counts alone, as a baseline keeps them.
    pub fn values(&self) -> BTreeMap<String, BTreeMap<String, u64>> {
        self.sections
            .iter()
            .map(|(section, metrics)| {
                let values = metrics.iter().map(|(n, m)| (n.clone(), m.value));
                (section.clone(), values.collect())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_a_metric_again_replaces_it() {
        // Arrange
        let mut report = Report::new();
        report.set("open", "place", 3, Better::Lower);

        // Act
        report.set("open", "place", 2, Better::Lower);

        // Assert
        assert_eq!(report.values()["open"]["place"], 2);
        assert_eq!(report.get("open", "place").map(|m| m.value), Some(2));
    }
}
