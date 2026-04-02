//! Argument validation for CLI.
//!
//! This module provides utilities for detecting formats from file extensions
//! and checking if paths are directories.

use std::path::Path;

/// Check if a path is a directory.
pub fn is_directory_path(path: &Path) -> bool {
    path.is_dir()
}

/// Detect format from file extension.
///
/// Maps file extension to format name for both readers and writers.
pub fn detect_format_from_extension(filename: &str, is_reader: bool) -> Option<String> {
    let ext = std::path::Path::new(filename).extension().map(|e| e.to_string_lossy().to_lowercase())?;

    if is_reader {
        // Reader format detection
        match ext.as_str() {
            "nw" | "nwchem" => Some("nwchem".to_string()),
            "gbs" => Some("gaussian94".to_string()),
            "tm" | "turbomole" => Some("turbomole".to_string()),
            "mol" | "dalton" => Some("dalton".to_string()),
            "molcas" => Some("molcas".to_string()),
            "mpro" | "molpro" => Some("molpro".to_string()),
            "genbas" | "c4bas" | "acesii" => Some("genbas".to_string()),
            "gms" | "gamess" => Some("gamess_us".to_string()),
            "cp2k" => Some("cp2k".to_string()),
            "crystal" => Some("crystal".to_string()),
            "libmol" => Some("libmol".to_string()),
            "gbas" | "gbasis" => Some("gbasis".to_string()),
            "dmon" | "demon2k" => Some("demon2k".to_string()),
            "vx" | "vlx" | "veloxchem" => Some("veloxchem".to_string()),
            "ricd" => Some("ricdlib".to_string()),
            "json" => Some("json".to_string()),
            _ => None,
        }
    } else {
        // Writer format detection
        match ext.as_str() {
            "nw" | "nwchem" => Some("nwchem".to_string()),
            "gbs" => Some("gaussian94".to_string()),
            "molcas" => Some("molcas".to_string()),
            "qchem" => Some("qchem".to_string()),
            "orca" => Some("orca".to_string()),
            "dalton" | "mol" => Some("dalton".to_string()),
            "json" => Some("qcschema".to_string()),
            "cp2k" => Some("cp2k".to_string()),
            "pqs" => Some("pqs".to_string()),
            "d2k" | "dmon" => Some("demon2k".to_string()),
            "bas" => Some("gamess_us".to_string()),
            "tm" => Some("turbomole".to_string()),
            "mpro" => Some("molpro".to_string()),
            "c4bas" => Some("cfour".to_string()),
            "bdf" => Some("bdf".to_string()),
            "fhiaims" => Some("fhiaims".to_string()),
            "jaguar" => Some("jaguar".to_string()),
            "crystal" => Some("crystal".to_string()),
            "vlx" | "vx" => Some("veloxchem".to_string()),
            _ => None,
        }
    }
}
