//! Conversion of references to EndNote format.

use crate::fields::BseReferenceEntry;

/// Convert a single reference to EndNote format (.enw).
///
/// EndNote is a commercial reference management software package.
/// The .enw format is a tagged text format for importing references.
///
/// # Arguments
///
/// * `key` - Reference key (e.g., "pritchard2019a")
/// * `ref_entry` - Reference data dictionary
///
/// # Returns
///
/// EndNote entry string.
pub fn write_endnote(key: &str, ref_entry: &BseReferenceEntry) -> String {
    let mut s = String::new();

    // Type mapping for EndNote
    let ty = match ref_entry.entry_type.as_str() {
        "article" => "Journal Article",
        "misc" => "Generic",
        "unpublished" => "Unpublished",
        "incollection" => "Book",
        "phdthesis" => "Thesis",
        "techreport" => "Report",
        "dataset" => "Data Set",
        _ => "Generic",
    };

    s += &format!("#{} {}\n", ref_entry.entry_type, key);
    s += &format!("%0 {} \n", ty);

    for (k, v) in &ref_entry.fields {
        match k.as_str() {
            "authors" => {
                for author in v.to_vec() {
                    s += &format!("%A {}\n", author);
                }
            },
            "year" => {
                if let Some(val) = v.first() {
                    s += &format!("%D {}\n", val);
                }
            },
            "journal" => {
                if let Some(val) = v.first() {
                    s += &format!("%J {}\n", val);
                }
            },
            "volume" => {
                if let Some(val) = v.first() {
                    s += &format!("%V {}\n", val);
                }
            },
            "pages" => {
                if let Some(val) = v.first() {
                    s += &format!("%P {}\n", val);
                }
            },
            "title" => {
                if let Some(val) = v.first() {
                    s += &format!("%T {}\n", val);
                }
            },
            "doi" => {
                if let Some(val) = v.first() {
                    s += &format!("%R {}\n", val);
                }
            },
            _ => {
                if let Some(val) = v.first() {
                    s += &format!("%Z {}:{}\n", k, val);
                }
            },
        }
    }

    s
}
