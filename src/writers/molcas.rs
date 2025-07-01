//! Conversion of basis sets to Molcas format.

use crate::prelude::*;

/// Converts a basis set to Molcas format.
pub fn write_molcas(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
    manip::prune_basis(&mut basis);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    for (z, data) in basis.elements.iter().sorted_by_key(|(z, _)| z.parse::<i32>().unwrap_or(0)) {
        s.push("Basis set".to_string());

        let has_electron = data.electron_shells.is_some();
        let has_ecp = data.ecp_potentials.is_some();

        let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        let el_sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
        s.push(format!(
            "* {}  {}",
            el_name,
            data.electron_shells.as_ref().map_or("".to_string(), |shls| misc::contraction_string(shls, HIK, INCOMPACT))
        ));

        // if ECP is present, the line should be "{sym}.ECP /inline"
        let ecp_tag = if has_ecp { ".ECP" } else { "" };
        s.push(format!(" {el_sym}{ecp_tag}    / inline"));

        if has_electron {
            let shells = data.electron_shells.as_ref().unwrap();
            let max_am = misc::max_am(shells);

            // number of electrons
            // should be z - number of ecp electrons
            let mut nelectrons = z.parse::<i32>().unwrap();
            if has_ecp {
                nelectrons -= data.ecp_electrons.unwrap();
            }

            s.push(format!("{nelectrons:>7}.00   {max_am}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let nprim = exponents.len();
                let ngen = coefficients.len();

                let amchar = lut::amint_to_char(&shell.angular_momentum, HIK).to_uppercase();
                s.push(format!("* {amchar}-type functions"));
                s.push(format!("{nprim:>6}    {ngen}"));

                s.push(printing::write_matrix(std::slice::from_ref(exponents), &[17], SCIFMT_E));

                let point_places = (1..=ngen).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                s.push(printing::write_matrix(coefficients, &point_places, SCIFMT_E));
            }
        }

        if has_ecp {
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("PP, {}, {}, {} ;", el_sym, data.ecp_electrons.unwrap(), max_ecp_am));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents;
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK);

                if am[0] == max_ecp_am {
                    s.push(format!("{}; !  ul potential", rexponents.len()));
                } else {
                    s.push(format!("{}; !  {amchar}-ul potential", rexponents.len()));
                }

                for p in 0..rexponents.len() {
                    s.push(format!("{},{},{};", rexponents[p], gexponents[p], coefficients[0][p]));
                }
            }

            s.push("Spectral".to_string());
            s.push("End of Spectral".to_string());
            s.push("*".to_string());
        }

        if has_electron {
            // Are there cartesian shells?
            let mut cartesian_shells = HashSet::new();
            for shell in data.electron_shells.as_ref().unwrap() {
                if shell.function_type == "gto_cartesian" {
                    for am in &shell.angular_momentum {
                        cartesian_shells.insert(lut::amint_to_char(&[*am], HIK));
                    }
                }
            }
            if !cartesian_shells.is_empty() {
                s.push(format!("cartesian {}", cartesian_shells.iter().join(" ")));
            }
        }

        s.push("End of basis set".to_string());
        s.push("".to_string()); // extra newline
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_molcas() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_molcas(&basis);
        println!("{output}");
    }
}
