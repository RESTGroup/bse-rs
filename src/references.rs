//! Helper functions for handling basis set references/citations.
//!
//! This module provides functions for compacting reference data from basis sets
//! and formatting individual references as plain text.

use crate::fields::{BseBasis, BseBasisElement, BseElementReferences, BseReferenceEntry, BseReferenceInfoWithData};
use itertools::Itertools;
use std::collections::HashMap;

/// Creates a mapping of elements to reference keys.
///
/// Given a basis set dictionary and all reference data, this function
/// creates a compacted list of element groups with their associated
/// references. Elements that share the same references are grouped together.
///
/// # Arguments
///
/// * `basis_dict` - Complete basis set information (from [`get_basis`])
/// * `ref_data` - All reference data from [`get_reference_data`]
///
/// # Returns
///
/// A list of [`BseElementReferences`] where each entry groups elements that
/// share the same reference information.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let all_ref_data = get_reference_data(None);
/// let basis = get_basis("cc-pVTZ", BseGetBasisArgs::default());
/// let compacted = compact_references(&basis, &all_ref_data);
/// for group in compacted {
///     println!("Elements: {}", bse::misc::compact_elements(&group.elements));
/// }
/// ```
pub fn compact_references(
    basis_dict: &BseBasis,
    ref_data: &HashMap<String, BseReferenceEntry>,
) -> Vec<BseElementReferences> {
    let mut element_refs: Vec<BseElementReferences> = Vec::new();

    // Create a mapping of elements -> reference information
    // Sort by Z (atomic number)
    let sorted_el: Vec<(i32, &BseBasisElement)> = basis_dict
        .elements
        .iter()
        .sorted_by_key(|(k, _)| k.parse::<i32>().unwrap_or(0))
        .map(|(k, v)| (k.parse::<i32>().unwrap_or(0), v))
        .collect();

    for (el, eldata) in sorted_el {
        // elref is a list of BseBasisReference (from the basis set element data)
        let elref = &eldata.references;

        // Check if we already have a group with the same reference info
        let found = element_refs.iter_mut().find(|x| {
            // Compare reference_info lists
            x.reference_info.len() == elref.len()
                && x.reference_info.iter().zip(elref.iter()).all(|(a, b)| {
                    a.reference_description == b.reference_description
                        && a.reference_data.iter().map(|(k, _)| k).collect::<Vec<_>>()
                            == b.reference_keys.iter().collect::<Vec<_>>()
                })
        });

        if let Some(group) = found {
            group.elements.push(el);
        } else {
            // Create new group
            let ref_info: Vec<BseReferenceInfoWithData> = elref
                .iter()
                .map(|r| BseReferenceInfoWithData {
                    reference_description: r.reference_description.clone(),
                    reference_data: r
                        .reference_keys
                        .iter()
                        .filter_map(|k| ref_data.get(k).map(|v| (k.clone(), v.clone())))
                        .collect(),
                })
                .collect();

            element_refs.push(BseElementReferences { reference_info: ref_info, elements: vec![el] });
        }
    }

    element_refs
}

/// Convert a single reference to plain text format.
///
/// Creates a human-readable plain text representation of a reference,
/// suitable for printing or inclusion in documentation.
///
/// # Arguments
///
/// * `key` - Reference key (e.g., "pritchard2019a")
/// * `ref_entry` - Reference entry data
///
/// # Returns
///
/// Plain text string with the key on the first line and formatted
/// reference details indented below.
pub fn reference_text(key: &str, ref_entry: &BseReferenceEntry) -> String {
    let indent = "        ";
    let width = 70;

    // Helper function to wrap text
    let wrap = |text: &str| -> String {
        textwrap::wrap(text, textwrap::Options::new(width).subsequent_indent(indent)).join("\n")
    };

    let mut s = String::new();

    match ref_entry.entry_type.as_str() {
        "unpublished" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            if let Some(title) = ref_entry.get_field_opt("title") {
                s += &wrap(&title);
                s += "\n";
            }
            if let Some(year) = ref_entry.get_field_opt("year") {
                s += &year;
                s += ", ";
            }
            s += "unpublished";
        },
        "article" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            s += "\n";
            s += &format!(
                "{} {}, {} ({})",
                ref_entry.get_field("journal").join(""),
                ref_entry.get_field("volume").join(""),
                ref_entry.get_field("pages").join(""),
                ref_entry.get_field("year").join("")
            );
            if let Some(doi) = ref_entry.get_field_opt("doi") {
                s += "\n";
                s += &doi;
            }
        },
        "incollection" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            s += "\n";
            s += &wrap(&format!("in '{}'", ref_entry.get_field("booktitle").join("")));
            if !ref_entry.get_field("editors").is_empty() {
                s += "\n";
                s += &wrap(&format!("ed. {}", ref_entry.get_field("editors").join(", ")));
            }
            if let Some(series) = ref_entry.get_field_opt("series") {
                s += "\n";
                s += &format!(
                    "{} {}, {} ({})",
                    series,
                    ref_entry.get_field("volume").join(""),
                    ref_entry.get_field("pages").join(""),
                    ref_entry.get_field("year").join("")
                );
            }
            if let Some(doi) = ref_entry.get_field_opt("doi") {
                s += "\n";
                s += &doi;
            }
        },
        "phdthesis" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            s += "\n";
            let thesis_type = ref_entry.get_field_opt("type").unwrap_or_else(|| "Ph.D. Thesis".to_string());
            s += &format!("{}, {}", thesis_type, ref_entry.get_field("school").join(""));
        },
        "techreport" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            s += "\n";
            s += &format!("'{}'", ref_entry.get_field("institution").join(""));
            s += "\n";
            let report_type = ref_entry.get_field_opt("type").unwrap_or_else(|| "Technical Report".to_string());
            if let Some(number) = ref_entry.get_field_opt("number") {
                s += &format!(" {} {}", report_type, number);
            } else {
                s += &format!(" {}", report_type);
            }
            s += &format!(", {}", ref_entry.get_field("year").join(""));
            if let Some(doi) = ref_entry.get_field_opt("doi") {
                s += "\n";
                s += &doi;
            }
        },
        "misc" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            if let Some(year) = ref_entry.get_field_opt("year") {
                s += "\n";
                s += &year;
            }
            if let Some(doi) = ref_entry.get_field_opt("doi") {
                s += "\n";
                s += &doi;
            }
        },
        "dataset" => {
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
            s += "\n";
            s += &format!("{} ({})", ref_entry.get_field("publisher").join(""), ref_entry.get_field("year").join(""));
            if let Some(doi) = ref_entry.get_field_opt("doi") {
                s += "\n";
                s += &doi;
            }
        },
        _ => {
            // Generic fallback
            s += &wrap(&ref_entry.get_field("authors").join(", "));
            s += "\n";
            s += &wrap(&ref_entry.get_field("title").join(""));
        },
    }

    if let Some(note) = ref_entry.get_field_opt("note") {
        s += "\n";
        s += &wrap(&note);
    }

    // The final output has the key on its own line. The rest is indented by 4
    let s_indented = s.lines().map(|line| format!("    {}", line)).join("\n");
    format!("{}\n{}", key, s_indented)
}
