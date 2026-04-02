//! Miscellaneous helper functions and constants.
//!
//! This module provides utility functions for element handling, formatting,
/// and string processing used throughout the crate.
use crate::prelude::*;

/* #region flags */

/// Angular momentum character ordering flag.
///
/// When `false` (HIK), uses ordering: spdfghikl (skips j).
/// When `true` (HIJ), uses ordering: spdfghijkl.
pub const HIK: bool = false;

/// Angular momentum character ordering flag (includes j).
pub const HIJ: bool = true;

/// Format flag for non-compact output.
pub const INCOMPACT: bool = false;

/// Format flag for compact output.
pub const COMPACT: bool = true;

/// Scientific notation format flag (uses 'e').
pub const SCIFMT_E: bool = false;

/// Scientific notation format flag (uses 'D' for Fortran compatibility).
pub const SCIFMT_D: bool = true;

/* #endregion */

/// Format a floating-point exponent in scientific notation.
///
/// Converts Rust's scientific notation to the format expected by BSE
/// (e.g., "1.234567e+02" format).
#[inline]
pub(crate) fn format_exponent(exp: f64) -> String {
    let token = format!("{exp:.6e}");
    let (s, e) = token.split_once('e').unwrap();
    let e = e.parse::<i32>().unwrap();
    let sgn = if e < 0 { '-' } else { '+' };
    let e = e.abs();
    format!("{s}e{sgn}{e:0>2}")
}

/// Transpose a matrix (list of lists).
///
/// Commonly used for converting between coefficient representations.
///
/// # Example
///
/// ```rust
/// use bse::misc::transpose_matrix;
/// let mat = vec![vec![1, 2], vec![3, 4]];
/// let transposed = transpose_matrix(&mat);
/// assert_eq!(transposed, vec![vec![1, 3], vec![2, 4]]);
/// ```
pub fn transpose_matrix<T>(mat: &[Vec<T>]) -> Vec<Vec<T>>
where
    T: Clone,
{
    if mat.is_empty() {
        return vec![];
    }

    let nrow = mat.len();
    let ncol = mat[0].len();

    (0..ncol).map(|i| (0..nrow).map(|j| mat[j][i].clone()).collect()).collect()
}

/// Determine the maximum angular momentum from a list of electron shells.
///
/// Returns the highest angular momentum value found, or 0 if no shells exist.
pub fn max_am(electron_shells: &[BseElectronShell]) -> i32 {
    electron_shells.iter().flat_map(|sh| &sh.angular_momentum).max().cloned().unwrap_or(0)
}

/// Form a string describing the contractions for an element.
///
/// Creates a human-readable description like "(16s,10p) -> \[4s,3p\]" or
/// the compact form "16s10p.4s3p".
///
/// # Arguments
///
/// * `electron_shells` - List of electron shells
/// * `hij` - Angular momentum character ordering (true = include j)
/// * `compact` - Output format (true = compact, false = expanded)
pub fn contraction_string(electron_shells: &[BseElectronShell], hij: bool, compact: bool) -> String {
    let mut cont_map: HashMap<i32, (usize, usize)> = HashMap::new();
    for sh in electron_shells {
        let nprim = sh.exponents.len();
        let ngeneral = sh.coefficients.len();

        // is a combined general contraction (sp, spd, etc)
        let is_spdf = sh.angular_momentum.len() > 1;

        for am in &sh.angular_momentum {
            // If this a general contraction (and not combined am), then use that
            let ncont = if !is_spdf { ngeneral } else { 1 };
            if !cont_map.contains_key(am) {
                cont_map.insert(*am, (nprim, ncont));
            } else {
                let (old_nprim, old_ncont) = cont_map.get_mut(am).unwrap();
                *old_nprim += nprim;
                *old_ncont += ncont;
            }
        }
    }

    let mut primstr = String::new();
    let mut contstr = String::new();
    for (&am, (nprim, ncont)) in cont_map.iter().sorted() {
        if am != 0 && !compact {
            primstr.push(',');
            contstr.push(',');
        }
        let amint_char = lut::amint_to_char(&[am], hij);
        primstr.push_str(&format!("{nprim}{amint_char}"));
        contstr.push_str(&format!("{ncont}{amint_char}"));
    }

    match compact {
        COMPACT => format!("{primstr}.{contstr}"),
        INCOMPACT => format!("({primstr}) -> [{contstr}]"),
    }
}

/// Create a compact string representation of element numbers.
///
/// Converts a list of atomic numbers into a human-readable string with
/// element symbols and ranges.
///
/// # Example
///
/// ```rust
/// use bse::misc::compact_elements;
/// assert_eq!(compact_elements(&[1, 2, 3, 6, 7, 8, 10]), "h-li,c-o,ne");
/// ```
pub fn compact_elements(elements: &[i32]) -> String {
    if elements.is_empty() {
        return String::new();
    }

    // Just to be safe, sort the list
    let mut elements = elements.iter().cloned().collect_vec();
    elements.sort_unstable();

    let mut ranges: Vec<Vec<i32>> = vec![];
    let mut i = 0;
    while i < elements.len() {
        let start = elements[i];
        while i + 1 < elements.len() && elements[i + 1] == elements[i] + 1 {
            i += 1;
        }

        let end = elements[i];
        if start == end {
            ranges.push(vec![start]);
        } else {
            ranges.push(vec![start, end]);
        }

        i += 1;
    }

    // Convert to elemental symbols
    ranges
        .iter()
        .map(|range| {
            if range.len() == 1 {
                lut::element_sym_from_Z(range[0]).unwrap().to_string()
            } else {
                let start_sym = lut::element_sym_from_Z(range[0]).unwrap();
                let end_sym = lut::element_sym_from_Z(range[1]).unwrap();
                if range[1] == range[0] + 1 {
                    format!("{start_sym},{end_sym}")
                } else {
                    format!("{start_sym}-{end_sym}")
                }
            }
        })
        .join(",")
}

/// Parse an element specification string into atomic numbers.
///
/// Accepts various formats including atomic numbers, element symbols,
/// and ranges. Case insensitive.
///
/// # Supported Formats
///
/// - Atomic numbers: "1, 2, 3" or just "1-3"
/// - Element symbols: "H, He, Li"
/// - Ranges: "H-Li", "1-3", "C-O"
/// - Combinations: "H-N,8,Na-12"
///
/// # Example
///
/// ```rust
/// use bse::misc::expand_elements;
/// assert_eq!(expand_elements("H-Li,C-O,Ne"), vec![1, 2, 3, 6, 7, 8, 10]);
/// assert_eq!(expand_elements("H-N,8,Na-12"), vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12]);
/// ```
pub fn expand_elements(compact_el: &str) -> Vec<i32> {
    expand_elements_f(compact_el).unwrap()
}

pub fn expand_elements_f(compact_el: &str) -> Result<Vec<i32>, BseError> {
    // - Remove brackets, quotes, and spaces
    let compact_el = compact_el.replace(['[', ']', '"', '\'', '(', ')'], "").to_lowercase();
    let compact_el = Regex::new(r",+").unwrap().replace_all(&compact_el, ",");
    let compact_el = Regex::new(r"-+").unwrap().replace_all(&compact_el, "-");
    let compact_el = Regex::new(r"\s+").unwrap().replace_all(&compact_el, "");
    let compact_el = compact_el.split(',').filter(|s| !s.is_empty()).collect_vec();

    let mut elements = HashSet::new();
    for el in compact_el {
        let dash_count = el.matches('-').count();
        match dash_count {
            0 => {
                let z = lut::element_Z_from_str(el).map_or(bse_raise!(ValueError, "Invalid element: {el}"), Ok)?;
                elements.insert(z);
            },
            1 => {
                if el.ends_with('-') {
                    // This is a range like "H-", which is invalid
                    bse_raise!(ValueError, "Invalid element range: {el}")?;
                } else if let Some(el_stop) = el.strip_prefix('-') {
                    let z =
                        lut::element_Z_from_str(el_stop).map_or(bse_raise!(ValueError, "Invalid element: {el}"), Ok)?;
                    let e = (1..=z).collect_vec();
                    elements.extend(e);
                } else {
                    let el_parts = el.split('-').collect_vec();
                    let el_start = el_parts[0];
                    let el_stop = el_parts[1];
                    let z_start = lut::element_Z_from_str(el_start)
                        .map_or(bse_raise!(ValueError, "Invalid element: {el_start}"), Ok)?;
                    let z_stop = lut::element_Z_from_str(el_stop)
                        .map_or(bse_raise!(ValueError, "Invalid element: {el_stop}"), Ok)?;
                    if z_start > z_stop {
                        bse_raise!(ValueError, "Invalid element range: {el}")?;
                    }
                    let e = (z_start..=z_stop).collect_vec();
                    elements.extend(e);
                }
            },
            _ => bse_raise!(ValueError, "Invalid element range: {el}")?,
        }
    }

    let elements = elements.into_iter().sorted().collect_vec();
    Ok(elements)
}

/// Transform a basis set name to its internal representation.
///
/// Converts to lowercase and replaces special characters for consistent
/// comparison and lookup.
pub fn transform_basis_name(name: &str) -> String {
    let mut transformed = name.to_lowercase();
    transformed = transformed.replace('/', "_sl_");
    transformed = transformed.replace('*', "_st_");
    transformed
}

/// Find the range of non-zero coefficients in a vector.
///
/// Returns (first_index, last_index) of non-zero values.
pub fn find_range(coeffs: &[String]) -> (usize, usize) {
    let non_zero: Vec<bool> = coeffs.iter().map(|x| x.parse::<f64>().unwrap() != 0.0).collect();

    // Find first non-zero index
    let first = non_zero.iter().position(|&x| x).unwrap();

    // Find last non-zero index
    let last = non_zero.iter().rposition(|&x| x).unwrap();

    (first, last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_elements() {
        assert_eq!(expand_elements("H-Li,C-O,Ne"), vec![1, 2, 3, 6, 7, 8, 10]);
        assert_eq!(expand_elements("H-N,8,Na-12"), vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12]);
        assert_eq!(expand_elements("[C,Al-15,S,17,'18']"), vec![6, 13, 14, 15, 16, 17, 18]);
    }
}
