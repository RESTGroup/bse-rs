//! Argument validation for CLI.
//!
//! This module provides utilities for detecting formats from file extensions
//! and checking if paths are directories.

use std::path::Path;

use crate::prelude::*;

/// Check if a path is a directory.
pub fn is_directory_path(path: &Path) -> bool {
    path.is_dir()
}

/// Detect format from file extension.
///
/// Uses the extension lookup maps defined in reader/writer modules
/// to automatically detect format from file extension.
pub fn detect_format_from_extension(filename: &str, is_reader: bool) -> Option<String> {
    let ext = Path::new(filename).extension().map(|e| e.to_string_lossy().to_lowercase())?;

    if is_reader {
        get_reader_format_by_extension(&ext).map(|s| s.to_string())
    } else {
        get_writer_format_by_extension(&ext).map(|s| s.to_string())
    }
}

/// Detect format from file extensions inside a directory.
///
/// Reads the directory contents and looks at file extensions to infer
/// the underlying format. Returns "dir-`<format>`" if a consistent format
/// is found across all recognized files.
///
/// # Arguments
///
/// * `dir_path` - Path to the directory
/// * `is_reader` - If true, use reader extension map; otherwise use writer
///
/// # Returns
///
/// * `Some("dir-<format>")` if a consistent format is detected
/// * `None` if directory is empty, has mixed formats, or unrecognized
///   extensions
///
/// # Example
///
/// ```rust,no_run
/// use bse::cli::check::detect_dir_format_from_files;
/// use std::path::Path;
///
/// // Directory with H.json, C.json, O.json files
/// let fmt = detect_dir_format_from_files(Path::new("/path/to/basis-dir"), true);
/// assert_eq!(fmt, Some("dir-json".to_string()));
/// ```
pub fn detect_dir_format_from_files(dir_path: &Path, is_reader: bool) -> Option<String> {
    if !dir_path.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(dir_path).ok()?;
    let mut detected_formats: Vec<String> = Vec::new();

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Get extension
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase())?;

        // Try to detect format from extension
        let fmt = if is_reader {
            get_reader_format_by_extension(&ext).map(|s| s.to_string())
        } else {
            get_writer_format_by_extension(&ext).map(|s| s.to_string())
        };

        if let Some(f) = fmt {
            detected_formats.push(f);
        }
    }

    // Need at least one recognized file
    if detected_formats.is_empty() {
        return None;
    }

    // Check if all detected formats are the same
    let first_format = &detected_formats[0];
    if detected_formats.iter().all(|f| f == first_format) {
        Some(format!("dir-{}", first_format))
    } else {
        // Mixed formats - cannot auto-detect
        None
    }
}
