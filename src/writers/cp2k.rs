//! Conversion of basis sets to cp2k format

use crate::prelude::*;

/// Converts a basis set to cp2k format
pub fn write_cp2k(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
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
            let elname = lut::element_name_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let cont_string = misc::contraction_string(shells, HIK, INCOMPACT);

            s.push(format!("# {elname} {} {cont_string}", basis.name));
            s.push(format!("{sym} {}", basis.name));

            let nshells = shells.len();
            s.push(format!("    {nshells}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let am = &shell.angular_momentum;
                let min_am = am.iter().min().unwrap();
                let max_am = am.iter().max().unwrap();
                let ncont = coefficients.len();
                let ncol = ncont + 1;
                let nprim = exponents.len();

                // First number is principle quantum number
                // But is not used, according to the documentation
                let mut shell_line = format!("1 {min_am} {max_am} {nprim}");

                if am.len() > 1 {
                    for _ in am {
                        shell_line.push_str(" 1");
                    }
                } else {
                    shell_line.push_str(&format!(" {ncont}"));
                }
                s.push(shell_line);

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
            s.push(String::new());
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        let bsname = basis.name.replace(' ', "_") + "_ECP";
        s.push("\n\n## Effective core potentials".to_string());
        s.push(bsname.clone());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("{sym} nelec {}", data.ecp_electrons.unwrap()));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;
                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_uppercase();

                if am[0] == max_ecp_am {
                    s.push(format!("{sym} ul"));
                } else {
                    s.push(format!("{sym} {amchar}"));
                }

                let point_places = [0, 9, 32];
                let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
        }
        s.push(format!("END {bsname}"));
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_cp2k() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_cp2k(&basis);
        println!("{output}");
    }
}
