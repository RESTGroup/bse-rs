//! Helper functions for reference conversion.

use crate::fields::BseReferenceEntry;
use std::collections::HashMap;

/// Library citation reference keys.
///
/// These reference keys should be cited when using the Basis Set Exchange
/// library or data.
static LIB_REFS: &[&str] = &["pritchard2019a", "feller1996a", "schuchardt2007a"];

/// Library citation description.
static LIB_REFS_DESC: &str = "If you downloaded data from the basis set
exchange or used the basis set exchange python library, please cite:\n";

/// Return a descriptive string and reference data for citing the BSE library.
///
/// When using the Basis Set Exchange library or data, users should cite
/// these three key papers.
///
/// # Arguments
///
/// * `all_ref_data` - All reference data from REFERENCES.json
///
/// # Returns
///
/// A tuple of (description string, reference data map).
pub fn get_library_citation(
    all_ref_data: &HashMap<String, BseReferenceEntry>,
) -> (String, HashMap<String, BseReferenceEntry>) {
    let lib_refs_data: HashMap<String, BseReferenceEntry> =
        LIB_REFS.iter().filter_map(|k| all_ref_data.get(*k).map(|v| ((*k).to_string(), v.clone()))).collect();

    (LIB_REFS_DESC.to_string(), lib_refs_data)
}
