//! Conversion of basis sets to Q-Chem format.

use crate::prelude::*;
use itertools::Itertools;

/// Determines the PURECART value for Q-Chem
fn determine_pure(basis: &BseBasis) -> String {
    // starts at d shells
    let mut pure = HashMap::new();
    for eldata in basis.elements.values() {
        let Some(shells) = &eldata.electron_shells else {
            continue;
        };
        for sh in shells {
            for shell_am in &sh.angular_momentum {
                let harm = if sh.function_type.contains("spherical") {
                    "1"
                } else {
                    "2" // cartesian
                };

                pure.entry(shell_am)
                    .and_modify(|e| {
                        if harm == "1" {
                            *e = "1";
                        }
                    })
                    .or_insert(harm);
            }
        }
    }

    let mut pure_list: Vec<_> = pure.into_iter().sorted_by(|a, b| b.0.cmp(a.0)).collect();
    // ECP has no pure_list, and truncation by `pure_list.len() - 2` will panic on
    // underflow
    pure_list.truncate(pure_list.len().max(2) - 2); // Trim s & p
    pure_list.into_iter().map(|x| x.1).collect()
}

/// Converts a basis set to Q-Chem format
///
/// Q-Chem is basically gaussian format, wrapped in $basis/$end
///
/// This also outputs the PURECART variable of the $rem block
pub fn write_qchem(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    s.push("$rem".to_string());
    if basis.role == "orbital" {
        if !electron_elements.is_empty() {
            s.push("    BASIS GEN".to_string());
        }
        if !ecp_elements.is_empty() {
            s.push("    ECP GEN".to_string());
        }
        s.push(format!("    PURECART {}", determine_pure(&basis)));
    } else {
        s.push("AUX_BASIS GEN".to_string());
    }
    s.push("$end".to_string());
    s.push("".to_string()); // empty line

    // Electron Basis
    if !electron_elements.is_empty() {
        let section = if basis.role == "orbital" { "basis" } else { "aux_basis" };
        s.push(format!("${section}"));

        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            s.push(format!("{sym}     0"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();
                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIJ).to_uppercase();

                s.push(format!("{amchar}   {nprim}   1.00"));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }
            s.push("****".to_string());
        }
        s.push("$end".to_string());
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n".to_string()); // empty line
        s.push("$ecp".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
            let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIJ);

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
                let amchar = lut::amint_to_char(am, HIJ);

                if am[0] == max_ecp_am {
                    s.push(format!("{amchar} potential"));
                } else {
                    s.push(format!("{amchar}-{max_ecp_amchar} potential"));
                }

                s.push(format!("  {nprim}"));

                let point_places = [0, 9, 32];
                let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }
            s.push("****".to_string());
        }
        s.push("$end".to_string());
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_qchem() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_qchem(&basis);
        println!("{output}");
    }
}
