//! Showing baselines as tables, one row per section.

use mosaico_metrics::Values;

/// `values` as a table: a row per section, under `first` as the heading
/// of the name column, and a column per metric in `columns`. A metric a
/// section does not have shows as `missing`.
pub fn table(values: &Values, first: &str, columns: &[&str], missing: &str) -> String {
    let mut rows = vec![
        std::iter::once(first.to_owned())
            .chain(columns.iter().map(|c| (*c).to_owned()))
            .collect::<Vec<_>>(),
    ];
    for (section, metrics) in values {
        let cells = columns.iter().map(|c| {
            metrics
                .get(*c)
                .map_or_else(|| missing.to_owned(), u64::to_string)
        });
        rows.push(std::iter::once(section.clone()).chain(cells).collect());
    }
    let widths: Vec<usize> = (0..=columns.len())
        .map(|i| rows.iter().map(|r| r[i].len()).max().unwrap_or(0))
        .collect();
    let mut out = String::new();
    for row in rows {
        let mut line = format!("{:<w$}", row[0], w = widths[0]);
        for (cell, width) in row.iter().zip(&widths).skip(1) {
            line.push_str(&format!("  {cell:>width$}"));
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_lines_up_its_columns_and_marks_missing_metrics() {
        // Arrange
        let mut values = Values::new();
        values
            .entry("core".into())
            .or_default()
            .insert("tests".into(), 1015);
        values
            .entry("cli".into())
            .or_default()
            .insert("tests".into(), 7);
        values
            .entry("cli".into())
            .or_default()
            .insert("unsafe".into(), 51);

        // Act
        let table = table(&values, "crate", &["tests", "unsafe"], "-");

        // Assert
        assert_eq!(
            table,
            "crate  tests  unsafe\n\
             cli        7      51\n\
             core    1015       -\n"
        );
    }
}
