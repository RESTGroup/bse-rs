//! Conversion of basis sets to BDF format.

use crate::prelude::*;

/// Converts a basis set to BDF format
pub fn write_bdf(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::make_general(&mut basis, INCOMPACT);
    manip::prune_basis(&mut basis);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Elements for which we have electron basis
    let electron_elements = basis
        .elements
        .iter()
        .filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k))
        .sorted_by_key(|z| z.parse::<i32>().unwrap())
        .collect_vec();

    // Elements for which we have ECP
    let ecp_elements = basis
        .elements
        .iter()
        .filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k))
        .sorted_by_key(|z| z.parse::<i32>().unwrap())
        .collect_vec();

    // Elements for which we have electron basis or ECP
    let all_elements = electron_elements
        .iter()
        .chain(ecp_elements.iter())
        .unique()
        .sorted_by_key(|z| z.parse::<i32>().unwrap())
        .collect_vec();

    if !electron_elements.is_empty() || !ecp_elements.is_empty() {
        for &z in all_elements {
            s.push("****".to_string());

            // Get element symbol
            let symbol = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let data = &basis.elements[z];

            if electron_elements.contains(&z) {
                let shells = data.electron_shells.as_ref().unwrap();
                let max_am = misc::max_am(shells);
                s.push(format!("{symbol}{z:>7}   {max_am}"));

                for shell in shells {
                    let exponents = &shell.exponents;
                    let coefficients = &shell.coefficients;
                    let nprim = exponents.len();
                    let ngen = coefficients.len();

                    let amchar = lut::amint_to_char(&shell.angular_momentum, HIK).to_uppercase();
                    s.push(format!("{amchar}    {nprim:>3}    {ngen}"));

                    s.push(printing::write_matrix(std::slice::from_ref(exponents), &[14], SCIFMT_E));

                    let point_places = (1..=ngen).map(|i| 7 + 20 * (i - 1)).collect_vec();
                    s.push(printing::write_matrix(coefficients, &point_places, SCIFMT_E));
                }
            }

            if ecp_elements.contains(&z) {
                s.push("ECP".to_string());

                let data = &basis.elements[z];
                let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
                let max_ecp_angular_momentum = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

                s.push(format!("{}     {}     {}", symbol, data.ecp_electrons.unwrap(), max_ecp_angular_momentum));

                // Sort lowest->highest, then put the highest at the beginning
                let mut ecp_list =
                    ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
                let ecp_list_last = ecp_list.pop().unwrap();
                ecp_list.insert(0, ecp_list_last);

                for pot in ecp_list {
                    let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                    let gexponents = &pot.gaussian_exponents;
                    let coefficients = &pot.coefficients;
                    let nprim = rexponents.len();

                    let am = &pot.angular_momentum;
                    let amchar = lut::amint_to_char(am, HIJ).to_uppercase();
                    s.push(format!("{amchar} potential  {nprim}"));

                    let point_places = [4, 12, 34];
                    let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                    s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
                }
            }
        }
    }

    s.push("****".to_string());

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_bdf() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_bdf(&basis);
        println!("{output}");
    }
}
