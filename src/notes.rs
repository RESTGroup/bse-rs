//! Helper functions for handling basis set and family notes.
//!
//! Notes files contain important information about basis sets and families,
//! including context, usage recommendations, and references. This module
//! provides functionality for reading and processing these notes.

use crate::fields::BseReferenceEntry;
use std::collections::HashMap;

/// Process notes by adding reference information at the bottom.
///
/// Reference keys mentioned in the notes (like `:ref:` tags in Python)
/// are identified and the actual reference data is appended to the notes.
///
/// # Arguments
///
/// * `notes` - The raw notes content
/// * `ref_data` - All reference data from [`crate::api::get_reference_data`]
///
/// # Returns
///
/// The notes with a formatted reference section appended, or the original
/// notes if no reference keys were found.
pub fn process_notes(notes: &str, ref_data: &HashMap<String, BseReferenceEntry>) -> String {
    // Find all reference keys that appear in the notes
    let ref_keys: Vec<&String> = ref_data.keys().collect();
    let mut found_refs: Vec<String> = Vec::new();

    for k in ref_keys {
        if notes.contains(k.as_str()) {
            found_refs.push(k.clone());
        }
    }

    // If no references found, return original notes
    if found_refs.is_empty() {
        return notes.to_string();
    }

    // Build the reference section
    let mut reference_sec = String::from("\n\n");
    reference_sec += "-------------------------------------------------\n";
    reference_sec += " REFERENCES MENTIONED ABOVE\n";
    reference_sec += " (not necessarily references for the basis sets)\n";
    reference_sec += "-------------------------------------------------\n";

    // Sort found references and add their data
    found_refs.sort();
    for r in &found_refs {
        if let Some(entry) = ref_data.get(r) {
            let rtxt = crate::references::reference_text(r, entry);
            reference_sec += &rtxt;
            reference_sec += "\n\n";
        }
    }

    notes.to_string() + &reference_sec
}

/// Read a notes file from disk.
///
/// Returns `None` if the file doesn't exist.
///
/// # Arguments
///
/// * `file_path` - Path to the notes file
pub fn read_notes_file(file_path: &str) -> Option<String> {
    let path = std::path::Path::new(file_path);
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(file_path).ok()
}
