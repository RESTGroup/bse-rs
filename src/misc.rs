//! Miscellaneous helper functions.

use crate::prelude::*;

/// Transposes a matrix (list of lists) commonly done do coefficients.
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

/// Forms a string specifying the contractions for an element.
///
/// i.e., (16s,10p) -> [4s,3p]
/// or, if compact=True, 16s10p.4s3p
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
    for (am, (nprim, ncont)) in cont_map {
        if am != 0 && !compact {
            primstr.push(',');
            contstr.push(',');
        }
        let amint_char = lut::amint_to_char(&[am], hij);
        primstr.push_str(&format!("{nprim}{amint_char}"));
        contstr.push_str(&format!("{ncont}{amint_char}"));
    }

    if compact { format!("{primstr}.{contstr}") } else { format!("({primstr}) -> [{contstr}]") }
}

/// Create a string (with ranges) given a list of element numbers.
///
/// For example, [1, 2, 3, 6, 7, 8, 10] will return "H-Li,C-O,Ne".
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

/// Create a list of integers given a string or list of compacted elements.
///
/// This is partly the opposite of compact_elements, but is more flexible.
///
/// In all cases, element symbols (case insensitive) and Z numbers (as integers
/// or strings) can be used interchangeably. Ranges are also allowed in both
/// lists and strings.
///
/// This function will ignore brackets (`[`, `]`), quotes (`'`, `"`) and spaces.
///
/// Some examples:
/// - "H-Li,C-O,Ne" will return [1, 2, 3, 6, 7, 8, 10]
/// - "H-N,8,Na-12" will return [1, 2, 3, 4, 5, 6, 7, 8, 11, 12]
/// - ['C', 'Al-15,S', 17, '18'] will return [6, 13, 14, 15, 16, 17, 18]
///
/// If as_str is True, the list will contain strings of the integers (i.e., the
/// first example above will return ['1', '2', '3', '6', '7', '8', '10'])
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

/// Transforms the name of a basis set to an internal representation.
///
/// This makes comparison of basis set names easier by, for example, converting
/// the name to all lower case.
pub fn transform_basis_name(name: &str) -> String {
    let mut transformed = name.to_lowercase();
    transformed = transformed.replace('/', "_sl_");
    transformed = transformed.replace('*', "_st_");
    transformed
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
