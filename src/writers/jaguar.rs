//! Conversion of basis sets to Jaguar format.

use crate::prelude::*;

/// Converts a basis set to Jaguar format
pub fn write_jaguar(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

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

    let types = &basis.function_types;
    let harm_type = if types.contains(&"gto_cartesian".to_string()) { "6D" } else { "5D" };
    let ecp_type = if !ecp_elements.is_empty() { " ECP" } else { "" };

    let mut s = vec![format!("BASIS {} {}{}", basis.name, harm_type, ecp_type)];

    // Electron Basis
    if !electron_elements.is_empty() {
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();

            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            s.push(sym.to_string());

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();

                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIJ).to_uppercase();

                s.push(format!("{amchar} 0 {nprim}"));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }

            if ecp_elements.contains(&z) {
                s.push("**".to_string());

                let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
                let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
                let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIK);

                // Sort lowest->highest, then put the highest at the beginning
                let mut ecp_list =
                    ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
                let ecp_list_last = ecp_list.pop().unwrap();
                ecp_list.insert(0, ecp_list_last);

                s.push(format!("{} {} {}", sym, max_ecp_am, data.ecp_electrons.unwrap()));

                for pot in ecp_list {
                    let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                    let gexponents = &pot.gaussian_exponents;
                    let coefficients = &pot.coefficients;

                    let am = &pot.angular_momentum;
                    let amchar = lut::amint_to_char(am, HIK);

                    if am[0] == max_ecp_am {
                        s.push(format!("{}_AND_UP", amchar.to_uppercase()));
                    } else {
                        s.push(format!("{}-{}", amchar.to_uppercase(), max_ecp_amchar.to_uppercase()));
                    }

                    let point_places = [0, 9, 32];
                    let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                    s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
                }
            }

            s.push("****".to_string());
        }
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_jaguar() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 8, 79".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_jaguar(&basis);
        println!("{output}");
    }
}
