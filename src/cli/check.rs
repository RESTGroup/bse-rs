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
