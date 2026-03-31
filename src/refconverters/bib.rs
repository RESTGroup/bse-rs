//! Conversion of references to BibTeX format.

use crate::fields::BseReferenceEntry;

/// Convert a single reference to BibTeX format.
///
/// # Arguments
///
/// * `key` - Reference key (e.g., "pritchard2019a")
/// * `ref_data` - Reference data dictionary
///
/// # Returns
///
/// BibTeX entry string.
pub fn write_bib(key: &str, ref_entry: &BseReferenceEntry) -> String {
    let mut s = String::new();

    s += &format!("@{}{{{},\n", ref_entry.entry_type, key);

    let mut entry_lines: Vec<String> = Vec::new();
    for (k, v) in &ref_entry.fields {
        if k == "authors" {
            entry_lines.push(format!("    author = {{{}}}", v.to_vec().join(" and ")));
        } else if k == "editors" {
            entry_lines.push(format!("    editor = {{{}}}", v.to_vec().join(" and ")));
        } else if let Some(val) = v.first() {
            entry_lines.push(format!("    {} = {{{}}}", k, val));
        }
    }

    s += &entry_lines.join(",\n");
    s += "\n}";

    s
}
