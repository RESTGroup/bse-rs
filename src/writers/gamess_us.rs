//! Conversion of basis sets to GAMESS-US.

use crate::prelude::*;

/// Writes the electron basis part for GAMESS-US format
fn write_gamess_us_electron_basis(basis: &BseBasis, electron_elements: &[&String]) -> String {
    let mut s = vec!["$DATA".to_string()];

    for z in electron_elements {
        let data = &basis.elements[*z];
        let shells = data.electron_shells.as_ref().unwrap();

        let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        s.push("".to_string());
        s.push(el_name);

        for shell in shells {
            let exponents = &shell.exponents;
            let coefficients = &shell.coefficients;
            let ncol = coefficients.len() + 2; // include index column
            let nprim = exponents.len();

            let am = &shell.angular_momentum;
            let amchar = lut::amint_to_char(am, HIJ).to_uppercase();
            s.push(format!("{amchar}   {nprim}"));

            // 1-based indexing
            let idx_column = (1..=nprim).collect_vec();
            let point_places = [0].into_iter().chain((1..ncol).map(|i| 4 + 8 * i + 15 * (i - 1))).collect_vec();

            let mut matrix_data = vec![idx_column.iter().map(|x| x.to_string()).collect_vec(), exponents.clone()];
            matrix_data.extend(coefficients.clone());
            s.push(printing::write_matrix(&matrix_data, &point_places, SCIFMT_E));
        }
    }

    // There must be a blank line before $END
    s.push("".to_string());
    s.push("$END".to_string());
    s.join("\n")
}

/// Writes the ECP basis part for GAMESS-US format
pub(crate) fn write_gamess_us_ecp_basis(basis: &BseBasis, ecp_elements: &[&String], ecp_block: bool) -> String {
    let mut s = vec![];

    if ecp_block {
        s.push("".to_string());
        s.push("".to_string());
        s.push("$ECP".to_string());
    }

    for z in ecp_elements {
        let data = &basis.elements[*z];
        let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
        let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
        let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIJ);

        // Sort lowest->highest, then put the highest at the beginning
        let mut ecp_list =
            ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
        let last_item = ecp_list.pop().unwrap();
        ecp_list.insert(0, last_item);

        s.push(format!("{}-ECP GEN    {}    {}", sym, data.ecp_electrons.unwrap(), max_ecp_am));

        for pot in ecp_list {
            let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
            let gexponents = &pot.gaussian_exponents;
            let coefficients = &pot.coefficients;
            let nprim = rexponents.len();

            let am = &pot.angular_momentum;
            let amchar = lut::amint_to_char(am, HIJ);

            // Title line
            if am[0] == max_ecp_am {
                s.push(format!("{nprim:<5} ----- {amchar}-ul potential -----"));
            } else {
                s.push(format!("{nprim:<5} ----- {amchar}-{max_ecp_amchar} potential -----"));
            }

            let point_places = [8, 23, 32];
            let mut matrix_data = coefficients.clone();
            matrix_data.push(rexponents.clone());
            matrix_data.push(gexponents.clone());
            s.push(printing::write_matrix(&matrix_data, &point_places, SCIFMT_E));
        }
    }

    if ecp_block {
        s.push("$END".to_string());
    }
    s.join("\n") + "\n"
}

/// Common conversion function for GAMESS-US format
pub(crate) fn write_gamess_us_common(basis: &BseBasis, ecp_func: impl Fn(&BseBasis, &[&String]) -> String) -> String {
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    let mut s = String::new();

    // Electron Basis
    if !electron_elements.is_empty() {
        s.push_str(&write_gamess_us_electron_basis(&basis, &electron_elements));
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push_str(&ecp_func(&basis, &ecp_elements));
    }
    s
}

/// Converts a basis set to GAMESS-US format
pub fn write_gamess_us(basis: &BseBasis) -> String {
    write_gamess_us_common(basis, |basis, ecp_elems| write_gamess_us_ecp_basis(basis, ecp_elems, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_gamess_us() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_gamess_us(&basis);
        println!("{output}");
    }
}
