//! Conversion of basis sets to Turbomole format

use crate::prelude::*;

/// Converts a basis set to Turbomole format
pub fn write_turbomole(basis: &BseBasis) -> String {
    // The role of the basis set determines what is put here
    let role = basis.role.as_str();

    // By default, we will just use '$basis', unless otherwise specified
    let mut s = vec![match role {
        "jfit" => "$jbas".to_string(),
        "jkfit" => "$jkbas".to_string(),
        "rifit" => "$cbas".to_string(),
        _ => "$basis".to_string(),
    }];
    s.push("*".to_string());

    // TM basis sets are completely uncontracted
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 0);
    sort::sort_basis(&mut basis);

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
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap();
            s.push(format!("{} {}", sym, basis.name));
            s.push("*".to_string());

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();

                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK);
                s.push(format!("    {nprim}   {amchar}"));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }
            s.push("*".to_string());
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("$ecp".to_string());
        s.push("*".to_string());
        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap();
            s.push(format!("{} {}-ecp", sym, basis.name));
            s.push("*".to_string());

            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
            let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIK);

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("  ncore = {}   lmax = {}", data.ecp_electrons.unwrap(), max_ecp_am));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK);

                if am[0] == max_ecp_am {
                    s.push(amchar.to_string());
                } else {
                    s.push(format!("{amchar}-{max_ecp_amchar}"));
                }

                let point_places = [9, 23, 32];
                let exp_coef = [coefficients.clone(), vec![rexponents.clone(), gexponents.clone()]].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }
            s.push("*".to_string());
        }
    }

    s.push("$end".to_string());
    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_turbomole() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_turbomole(&basis);
        println!("{output}");
    }
}
