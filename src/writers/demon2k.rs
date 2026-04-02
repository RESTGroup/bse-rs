//! Conversion of basis sets to deMon2K format

use crate::prelude::*;

/// Converts a basis set to deMon2K format
pub fn write_demon2k(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    let mut s: Vec<String> = vec![];

    // Add comment about spherical/cartesian
    if basis.function_types.contains(&"gto_spherical".to_string()) {
        s.push("# This basis set uses spherical components\n".to_string());
    } else {
        s.push("# This basis set uses cartesian components\n".to_string());
    }

    // Uncontract basis
    manip::uncontract_spdf(&mut basis, 0);
    manip::uncontract_general(&mut basis);
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
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let elname = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let cont_string = misc::contraction_string(shells, HIK, INCOMPACT);

            // Need the start of electron shells if there are ECPs
            let ecp_electrons = data.ecp_electrons.unwrap_or(0);
            let mut shells_start = lut::electron_shells_start(ecp_electrons, 20);

            s.push(format!("O-{elname} {} ({})", sym.to_uppercase(), basis.name));
            s.push(format!("# {cont_string}"));
            s.push(format!("    {}", shells.len()));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();

                // We removed spdf already
                assert_eq!(shell.angular_momentum.len(), 1);
                let am = shell.angular_momentum[0];

                // shells_start has starting principal quantum numbers for all AM
                let pqn = shells_start[am as usize];
                shells_start[am as usize] += 1;
                s.push(format!("    {pqn}    {am}    {nprim}"));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n\nECP".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
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
        s.push("END".to_string());
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_demon2k() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_demon2k(&basis);
        println!("{output}");
    }
}
