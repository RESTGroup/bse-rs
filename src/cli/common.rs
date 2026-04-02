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

/// Format data as a bordered table with headers.
///
/// Creates a table with borders using Unicode box-drawing characters:
/// - `│` for vertical separators
/// - `─` for horizontal separators
/// - `┌`, `┐`, `└`, `┘` for corners
/// - `├`, `┤`, `┬`, `┴`, `┼` for intersections
///
/// # Arguments
///
/// * `headers` - Column headers
/// * `rows` - Table data, each row is a vec of strings
///
/// # Returns
///
/// A vector of formatted strings representing the table.
///
/// # Example
///
/// ```
/// use bse::cli::common::format_table;
/// let headers = vec!["Name", "Aliases", "Display"];
/// let rows = vec![
///     vec!["nwchem".to_string(), "nw".to_string(), "NWChem".to_string()],
///     vec!["gaussian94".to_string(), "g94, gbs".to_string(), "Gaussian".to_string()],
/// ];
/// let table = format_table(&headers, &rows);
/// // Returns bordered table with aligned columns
/// ```
pub fn format_table<S: AsRef<str>>(headers: &[S], rows: &[Vec<String>]) -> Vec<String> {
    if headers.is_empty() {
        return Vec::new();
    }

    let ncols = headers.len();

    // Calculate column widths (max of header and all rows)
    let col_widths: Vec<usize> = (0..ncols)
        .map(|c| {
            let header_width = headers[c].as_ref().len();
            let row_width = rows.iter().map(|r| r.get(c).map(|s| s.len()).unwrap_or(0)).max().unwrap_or(0);
            std::cmp::max(header_width, row_width)
        })
        .collect();

    // Helper to create horizontal separator
    let make_separator = |left: &str, _mid: &str, right: &str, cross: &str| -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.push(left.to_string());
        for (i, w) in col_widths.iter().enumerate() {
            parts.push("─".repeat(*w + 2)); // +2 for padding
            if i < ncols - 1 {
                parts.push(cross.to_string());
            }
        }
        parts.push(right.to_string());
        parts.join("")
    };

    let mut result = Vec::new();

    // Top border
    result.push(make_separator("┌", "─", "┐", "┬"));

    // Header row
    let mut header_parts: Vec<String> = Vec::new();
    header_parts.push("│".to_string());
    for (c, h) in headers.iter().enumerate() {
        let width = col_widths[c];
        header_parts.push(format!(" {:width$} ", h.as_ref(), width = width));
        header_parts.push("│".to_string());
    }
    result.push(header_parts.join(""));

    // Header/body separator
    result.push(make_separator("├", "─", "┤", "┼"));

    // Data rows
    for row in rows {
        let mut row_parts: Vec<String> = Vec::new();
        row_parts.push("│".to_string());
        for (c, val) in row.iter().enumerate() {
            let width = col_widths.get(c).copied().unwrap_or(0);
            // If value is longer than width (shouldn't happen), truncate
            let display_val = if val.len() > width { &val[..width] } else { val.as_str() };
            row_parts.push(format!(" {:width$} ", display_val, width = width));
            row_parts.push("│".to_string());
        }
        result.push(row_parts.join(""));
    }

    // Bottom border
    result.push(make_separator("└", "─", "┘", "┴"));

    result
}
