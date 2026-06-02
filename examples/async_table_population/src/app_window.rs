use std::path::Path;

use anyhow::{Context, Result};

/// Reads a CSV file and returns its headers and row data as owned strings.
pub fn load_csv_file(path: impl AsRef<Path>) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let path = path.as_ref();
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();

    let rows: Vec<Vec<String>> = rdr
        .records()
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|field| field.to_string()).collect())
        .collect();

    Ok((headers, rows))
}
