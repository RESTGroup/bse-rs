//! Common formatting utilities for CLI output.
//!
//! This module provides helpers for formatting CLI output in a consistent
//! column-aligned manner, similar to Python's `common.py`.

/// Format lines into aligned columns.
///
/// Takes a list of tuples/arrays and formats them into aligned columns.
/// Each column (except the last) is aligned to the maximum width of that
/// column.
///
/// # Arguments
///
/// * `lines` - List of lines to format, each line is a tuple/array of strings
/// * `prefix` - Characters to insert at the beginning of each line
///
/// # Returns
///
/// A vector of formatted strings, one per line.
///
/// # Example
///
/// ```
/// use bse::cli::common::format_columns;
/// let lines = vec![
///     vec!["nwchem", "NWChem format"],
///     vec!["gaussian94", "Gaussian format"],
/// ];
/// let formatted = format_columns(&lines, "");
/// // Returns:
/// // "nwchem       NWChem format"
/// // "gaussian94   Gaussian format"
/// ```
pub fn format_columns<S: AsRef<str>>(lines: &[Vec<S>], prefix: &str) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Find the number of columns
    let ncols = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    if ncols == 0 {
        return Vec::new();
    }

    // Find max length for each column (except last)
    let maxlen: Vec<usize> = (0..ncols.saturating_sub(1))
        .map(|c| lines.iter().map(|l| l.get(c).map(|s| s.as_ref().len()).unwrap_or(0)).max().unwrap_or(0))
        .collect();

    // Format each line
    lines
        .iter()
        .map(|l| {
            let mut parts: Vec<String> = Vec::new();
            for (c, s) in l.iter().enumerate() {
                if c < ncols - 1 {
                    // Pad with spaces to max length + 2 for spacing
                    let pad = maxlen.get(c).copied().unwrap_or(0).saturating_sub(s.as_ref().len()) + 2;
                    parts.push(format!("{}{}", s.as_ref(), " ".repeat(pad)));
                } else {
                    parts.push(s.as_ref().to_string());
                }
            }
            format!("{}{}", prefix, parts.join("").trim_end())
        })
        .collect()
}

/// Format a simple key-value listing as columns.
///
/// Takes a hashmap and formats it as two columns: key and value.
///
/// # Arguments
///
/// * `items` - HashMap or iterator of key-value pairs
/// * `prefix` - Characters to insert at the beginning of each line
///
/// # Returns
///
/// A vector of formatted strings, sorted by key.
pub fn format_map_columns<K: AsRef<str>, V: AsRef<str>>(items: &[(K, V)], prefix: &str) -> Vec<String> {
    let lines: Vec<Vec<String>> =
        items.iter().map(|(k, v)| vec![k.as_ref().to_string(), v.as_ref().to_string()]).collect();
    format_columns(&lines, prefix)
}
