//! Conversion of references to RIS format.

use crate::fields::BseReferenceEntry;

/// Convert a single reference to RIS format.
///
/// RIS is a standardized tag format commonly used by reference managers
/// like Reference Manager, EndNote, and ProCite.
///
/// # Arguments
///
/// * `key` - Reference key (e.g., "pritchard2019a")
/// * `ref_entry` - Reference data dictionary
///
/// # Returns
///
/// RIS entry string.
pub fn write_ris(key: &str, ref_entry: &BseReferenceEntry) -> String {
    let mut s = String::new();

    // Type mapping
    let ty = match ref_entry.entry_type.as_str() {
        "article" => "Journal Article",
        "misc" => "Generic",
        "unpublished" => "Unpublished",
        "incollection" => "Book",
        "phdthesis" => "Thesis",
        "dataset" => "Dataset",
        "techreport" => "Report",
        _ => "Generic",
    };

    s += &format!("#{} {}\n", ref_entry.entry_type, key);
    s += &format!("TY {} \n", ty);

    for (k, v) in &ref_entry.fields {
        match k.as_str() {
            "authors" => {
                for author in v.to_vec() {
                    s += &format!("AU {}\n", author);
                }
            },
            "year" => {
                if let Some(val) = v.first() {
                    s += &format!("PY {}\n", val);
                }
            },
            "journal" => {
                if let Some(val) = v.first() {
                    s += &format!("JO {}\n", val);
                }
            },
            "volume" => {
                if let Some(val) = v.first() {
                    s += &format!("VL {}\n", val);
                }
            },
            "pages" => {
                if let Some(val) = v.first() {
                    s += &format!("SP {}\n", val);
                }
            },
            "title" => {
                if let Some(val) = v.first() {
                    s += &format!("T1 {}\n", val);
                }
            },
            "doi" => {
                if let Some(val) = v.first() {
                    s += &format!("DO {}\n", val);
                }
            },
            _ => {
                if let Some(val) = v.first() {
                    s += &format!("N1 {}:{}\n", k, val);
                }
            },
        }
    }

    s
}
