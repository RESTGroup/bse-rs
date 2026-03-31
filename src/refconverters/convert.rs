//! Driver for converting basis set references to specified output formats.

use crate::fields::{BseElementReferences, BseReferenceEntry};
use crate::misc;
use crate::prelude::*;
use crate::refconverters::{bib, common, endnote, ris};
use crate::references::reference_text;
use itertools::Itertools;
use std::collections::HashMap;
use textwrap;

/// Converter format information.
struct ConverterFormat {
    display: &'static str,
    extension: &'static str,
    comment: &'static str,
}

/// Map of available reference formats.
fn converter_map() -> HashMap<&'static str, ConverterFormat> {
    HashMap::from([
        ("txt", ConverterFormat { display: "Plain Text", extension: ".txt", comment: "" }),
        ("bib", ConverterFormat { display: "BibTeX", extension: ".bib", comment: "%" }),
        ("ris", ConverterFormat { display: "RIS", extension: ".RIS", comment: "#" }),
        ("endnote", ConverterFormat { display: "EndNote", extension: ".enw", comment: "#" }),
        ("json", ConverterFormat { display: "JSON", extension: ".json", comment: "" }),
    ])
}

/// Return information about the reference/citation formats available.
///
/// The returned data is a map of format name to display name. The format
/// can be passed as the `fmt` argument to
/// [`get_references`][crate::api::get_references].
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let formats = get_reference_formats();
/// assert!(!formats.is_empty());
/// assert!(formats.contains_key("bib"));
/// println!("Available reference formats: {:?}", formats);
/// ```
pub fn get_reference_formats() -> HashMap<String, String> {
    converter_map().into_iter().map(|(k, v)| (k.to_string(), v.display.to_string())).collect()
}

/// Return the recommended file extension for a given reference format.
///
/// # Arguments
///
/// * `fmt` - The format name (case insensitive)
///
/// # Returns
///
/// The recommended file extension (e.g., ".bib" for BibTeX).
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let ext = get_reference_format_extension("bib").unwrap();
/// assert_eq!(ext, ".bib");
/// ```
pub fn get_reference_format_extension(fmt: &str) -> Result<&'static str, BseError> {
    let fmt = fmt.to_lowercase();
    let map = converter_map();
    if !map.contains_key(fmt.as_str()) {
        bse_raise!(ValueError, "Unknown reference format '{}'", fmt)?;
    }
    Ok(map[fmt.as_str()].extension)
}

/// Convert basis set references to a specified output format.
///
/// Takes the compacted reference data from a basis set and converts it
/// to a human-readable format for citing the basis set in publications.
///
/// # Arguments
///
/// * `ref_data` - Compacted reference data (list of element reference groups)
/// * `fmt` - Output format (bib, ris, endnote, txt, json)
/// * `all_ref_data` - All reference data from REFERENCES.json
///
/// # Returns
///
/// Formatted string containing all references for the basis set.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let all_ref_data = get_reference_data(None);
/// let basis = get_basis("cc-pVTZ", BseGetBasisArgs::default());
/// let ref_data = compact_references(&basis, &all_ref_data);
/// let bib_output = convert_references(&ref_data, "bib", &all_ref_data);
/// println!("{}", bib_output);
/// ```
pub fn convert_references(
    ref_data: &[BseElementReferences],
    fmt: &str,
    all_ref_data: &HashMap<String, BseReferenceEntry>,
) -> String {
    convert_references_f(ref_data, fmt, all_ref_data).unwrap()
}

pub fn convert_references_f(
    ref_data: &[BseElementReferences],
    fmt: &str,
    all_ref_data: &HashMap<String, BseReferenceEntry>,
) -> Result<String, BseError> {
    // Make fmt case insensitive
    let fmt = fmt.to_lowercase();
    let map = converter_map();
    if !map.contains_key(fmt.as_str()) {
        bse_raise!(ValueError, "Unknown reference format '{}'", fmt)?;
    }

    // Shortcut for JSON - just serialize the data
    if fmt == "json" {
        return Ok(serde_json::to_string_pretty(ref_data)?);
    }

    let comment = map[fmt.as_str()].comment;
    let comment_line = if comment.is_empty() { String::new() } else { format!("{}\n", comment.repeat(80)) };

    // Actually do the conversion
    let mut ref_str = String::new();

    // First, add library citations (for citing the BSE)
    let (lib_citation_desc, lib_citations) = common::get_library_citation(all_ref_data);

    if !comment.is_empty() {
        ref_str += &comment_line;
        ref_str += &textwrap::indent(&lib_citation_desc, &format!("{} ", comment));
        ref_str += &comment_line;

        for (k, r) in &lib_citations {
            ref_str += &format_single_ref(k, r, &fmt)?;
            ref_str += "\n\n";
        }

        ref_str += &comment_line;
        ref_str += &format!("{} References for the basis set\n", comment);
        ref_str += &comment_line;
    }

    // Build mapping and collect unique references
    let mut unique_refs: HashMap<String, BseReferenceEntry> = HashMap::new();

    for ref_item in ref_data {
        if !comment.is_empty() {
            ref_str += &format!("{} {}\n", comment, misc::compact_elements(&ref_item.elements));
        }

        for ri in &ref_item.reference_info {
            if !comment.is_empty() {
                ref_str += &format!("{}     {}\n", comment, ri.reference_description);

                if ri.reference_data.is_empty() {
                    ref_str += &format!("{}         (...no reference...)\n{}\n", comment, comment);
                } else {
                    let rkeys: Vec<String> = ri.reference_data.iter().map(|(k, _)| k.clone()).collect();
                    ref_str += &format!("{}         {}\n{}\n", comment, rkeys.join(" "), comment);
                }
            }

            for (k, r) in &ri.reference_data {
                unique_refs.insert(k.clone(), r.clone());
            }
        }
    }

    ref_str += "\n\n";

    // Go through unique refs sorted alphabetically by key
    for (k, r) in unique_refs.iter().sorted_by_key(|(k, _)| k.as_str()) {
        ref_str += &format_single_ref(k, r, &fmt)?;
        ref_str += "\n\n";
    }

    Ok(ref_str)
}

/// Format a single reference using the appropriate converter.
fn format_single_ref(key: &str, ref_entry: &BseReferenceEntry, fmt: &str) -> Result<String, BseError> {
    match fmt {
        "bib" => Ok(bib::write_bib(key, ref_entry)),
        "ris" => Ok(ris::write_ris(key, ref_entry)),
        "endnote" => Ok(endnote::write_endnote(key, ref_entry)),
        "txt" => Ok(reference_text(key, ref_entry)),
        _ => bse_raise!(ValueError, "Unknown format: {}", fmt),
    }
}
