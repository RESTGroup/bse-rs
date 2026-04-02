//! Conversion of basis sets to GAMESS-UK format

use crate::prelude::*;

/// Converts a basis set to GAMESS-UK format
pub fn write_gamess_uk(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    // Uncontract all but SP
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

    // Electron Basis
    if !electron_elements.is_empty() {
        // electronic part starts with $DATA
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();

            let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let el_sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            s.push(format!("\n# {el_name}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 2; // include index column

                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIJ).to_uppercase();
                s.push(format!("{amchar}   {el_sym}"));

                // 1-based indexing
                let point_places = (1..ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();

                // Note: order for sp shells is (coeff of s) (exponents) (coeff of p)
                let mut exp_coef = vec![coefficients[0].clone()];
                exp_coef.push(exponents.clone());
                exp_coef.extend(coefficients[1..].to_vec());
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n\nEffective Core Potentials".to_string());
        s.push("---------------------------".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("CARDS {sym}"));
            s.push(format!("    {max_ecp_am}     {}", data.ecp_electrons.unwrap()));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let point_places = [1, 9, 32];
                let mut exp_coef = vec![rexponents.clone()];
                exp_coef.extend(coefficients.clone());
                exp_coef.push(gexponents.clone());
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
            s.push("".to_string());
        }
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_gamess_uk() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_gamess_uk(&basis);
        println!("{output}");
    }
}
