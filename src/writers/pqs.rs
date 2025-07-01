//! Conversion of basis sets to PQS format

use crate::prelude::*;

/// Writes the electron basis part for PQS format
fn write_pqs_electron_basis(basis: &BseBasis, electron_elements: &[&String]) -> String {
    let mut s = vec![];

    for z in electron_elements {
        let data = &basis.elements[*z];
        let shells = data.electron_shells.as_ref().unwrap();

        let el_sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
        s.push(format!("FOR        {el_sym}"));

        for shell in shells {
            let exponents = &shell.exponents;
            let coefficients = &shell.coefficients;

            let am = &shell.angular_momentum;
            let amchar = lut::amint_to_char(am, HIJ).to_uppercase();

            let ncol = coefficients.len() + 1;
            let point_places = (1..=ncol).map(|i| 4 + 8 * i + 15 * (i - 1)).collect_vec();
            let mut mat = printing::write_matrix(
                &[vec![exponents.clone()], coefficients.clone()].concat(),
                &point_places,
                SCIFMT_E,
            );

            // Prepend the AM
            if !mat.is_empty() {
                mat = format!("{}{}", amchar, &mat[1..]);
            }
            s.push(mat);
        }
    }

    s.join("\n") + "\n"
}

/// Converts the basis set to PQS format
pub fn write_pqs(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::make_general(&mut basis, true);
    manip::prune_basis(&mut basis);
    sort::sort_basis(&mut basis);

    let mut s = String::new();

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    // Electron Basis
    if !electron_elements.is_empty() {
        s.push_str(&write_pqs_electron_basis(&basis, &electron_elements));
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push_str("\n\n");
        s.push_str("Effective core Potentials\n");
        s.push_str("-------------------------\n");
        s.push_str(&writers::gamess_us::write_gamess_us_ecp_basis(&basis, &ecp_elements, false));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_pqs() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_pqs(&basis);
        println!("{output}");
    }
}
