//! Miscellaneous helper functions.

/// Transforms the name of a basis set to an internal representation
///
/// This makes comparison of basis set names easier by, for example,
/// converting the name to all lower case.
pub fn transform_basis_name(name: &str) -> String {
    let mut transformed = name.to_lowercase();
    transformed = transformed.replace('/', "_sl_");
    transformed = transformed.replace('*', "_st_");
    transformed
}
