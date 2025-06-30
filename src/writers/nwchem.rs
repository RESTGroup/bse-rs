//! Conversion of basis sets to NWChem format.

use crate::prelude::*;

const HIK: bool = false;
const INCOMPACT: bool = false;
const SCIFMT_E: bool = false;

/// Converts a basis set to NWChem format.
pub fn write_nwchem(basis: &BseBasis) -> String {
    // Uncontract all but SP
    let mut basis = basis.clone();
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    if !electron_elements.is_empty() {
        // Angular momentum type
        let types = basis.function_types;
        let harm_type = if types.contains(&"gto_cartesian".to_string()) { "cartesian" } else { "spherical" };

        // basis set starts with a string
        s.push(format!(r#"BASIS "ao basis" {} PRINT"#, harm_type.to_uppercase()));

        // Electron Basis
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            s.push(format!(r#"#BASIS SET: {}"#, misc::contraction_string(shells, HIK, INCOMPACT)));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_uppercase();
                s.push(format!(r#"{sym}    {amchar}"#));

                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
        }
        s.push("END".to_string());
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n\nECP".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->hightest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!(r#"{sym} nelec {}"#, data.ecp_electrons.unwrap()));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_uppercase();

                if am[0] == max_ecp_am {
                    s.push(format!(r#"{sym} ul"#));
                } else {
                    s.push(format!(r#"{sym} {amchar}"#));
                }

                let point_places = [0, 10, 33];
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
    fn test_get_header() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_nwchem(&basis);
        println!("{output}");
    }
}
