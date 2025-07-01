//! Conversion of basis sets to Gaussian format

use crate::prelude::*;

/// Common function for writing Gaussian94-like formats
fn write_g94_common(basis: &BseBasis, add_harm_type: bool, psi4_am: bool, system_library: bool) -> String {
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 1);
    manip::prune_basis(&mut basis);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    // Electron Basis
    if !electron_elements.is_empty() {
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();

            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            if system_library {
                s.push(format!("-{sym}     0"));
            } else {
                s.push(format!("{sym}     0"));
            }

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();

                let am = &shell.angular_momentum;
                let mut amchar = lut::amint_to_char(am, HIJ).to_uppercase();

                if psi4_am && am.len() == 1 && am[0] >= 7 {
                    // For am=7 and above, use explicit L={am} notation
                    amchar = format!("L={}", am[0]);
                }

                let mut harm = String::new();
                if add_harm_type && shell.function_type == "gto_cartesian" {
                    harm = " c".to_string();
                }
                s.push(format!("{amchar:4} {nprim}   1.00{harm}"));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }

            s.push("****".to_string());
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("".to_string());
        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
            let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], true);

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("{sym}     0"));
            s.push(format!("{sym}-ECP     {max_ecp_am}     {}", data.ecp_electrons.unwrap()));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;
                let nprim = rexponents.len();

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, true);

                if am[0] == max_ecp_am {
                    s.push(format!("{amchar} potential"));
                } else {
                    s.push(format!("{amchar}-{max_ecp_amchar} potential"));
                }

                s.push(format!("  {nprim}"));

                let point_places = [0, 9, 32];
                let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, true));
            }
        }
    }

    s.join("\n") + "\n"
}

/// Converts a basis set to Gaussian format
pub fn write_g94(basis: &BseBasis) -> String {
    write_g94_common(basis, false, false, false)
}

/// Converts a basis set to Gaussian system library format
pub fn write_g94lib(basis: &BseBasis) -> String {
    write_g94_common(basis, false, false, true)
}

/// Converts a basis set to xTron format
///
/// xTron uses a modified gaussian format that puts 'c' on the same
/// line as the angular momentum if the shell is cartesian.
pub fn write_xtron(basis: &BseBasis) -> String {
    write_g94_common(basis, true, false, false)
}

/// Converts a basis set to Psi4 format
///
/// Psi4 uses the same output as gaussian94, except
/// that the first line must be cartesian/spherical,
/// and it prefers to have a starting asterisks
///
/// The cartesian/spherical line is added later, since it must
/// be the first non-blank line.
pub fn write_psi4(basis: &BseBasis) -> String {
    let mut s = String::new();
    s.push_str("****\n");
    s.push_str(&write_g94_common(basis, false, true, false));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_g94() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_g94(&basis);
        println!("{output}");
    }

    #[test]
    fn test_write_psi4() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_psi4(&basis);
        println!("{output}");
    }
}
