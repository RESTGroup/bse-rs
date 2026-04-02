//! Conversion of basis sets to Dalton format

use crate::prelude::*;

/// Converts a basis set to Dalton format
pub fn write_dalton(basis: &BseBasis) -> String {
    let mut s = vec![format!("! Basis = {}\n", basis.name)];

    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
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
            let elname = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let cont_string = misc::contraction_string(shells, HIJ, INCOMPACT);

            s.push(format!("a {z}"));
            s.push(format!("! {elname}       {cont_string}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();
                let ngen = coefficients.len();

                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIJ);
                s.push(format!("! {amchar} functions"));

                s.push(format!("H    {nprim}    {ngen}"));

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
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("a {:3}\n$", z.parse::<i32>().unwrap()));
            s.push(format!("{:4}{:4}", max_ecp_am, data.ecp_electrons.unwrap()));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                s.push(format!("{:12}", rexponents.len()));

                let point_places = [0, 9, 32];
                let exp_coef = [vec![rexponents.clone(), gexponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
            s.push("$".to_string());
        }
        s.push("$ END OF ECP".to_string());
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_dalton() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_dalton(&basis);
        println!("{output}");
    }
}
