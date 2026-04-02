//! Conversion of basis sets to FHI-aims format

use crate::prelude::*;

/// Converts a basis set to FHI-aims format
pub fn write_fhiaims(basis: &BseBasis) -> String {
    // Angular momentum type
    let types = &basis.function_types;
    let pure = !types.contains(&"gto_cartesian".to_string());

    // Set up
    let mut s: Vec<String> = vec![];
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 0);
    sort::sort_basis(&mut basis);

    // Elements for which we have electron basis
    let electron_elements = basis
        .elements
        .iter()
        .filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k))
        .sorted_by_key(|z| z.parse::<i32>().unwrap())
        .collect_vec();

    // Electron Basis
    if !electron_elements.is_empty() {
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();

            // FHI-aims defines elements in species default files.
            // The options need to be specified for every element basis.
            s.push("".to_string()); // Empty line for spacing
            s.push("            # The default minimal basis should not be included".to_string());
            s.push("            include_min_basis .false.".to_string());
            s.push("            # Use spherical functions?".to_string());
            s.push(format!("            pure_gauss {}", if pure { ".true." } else { ".false." }));

            let sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();
            s.push(format!("# {sym} {}", basis.name));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();
                let am = &shell.angular_momentum;
                assert_eq!(am.len(), 1);

                if nprim == 1 {
                    s.push(format!("gaussian {} {} {}", am[0], nprim, exponents[0]));
                } else {
                    s.push(format!("gaussian {} {}", am[0], nprim));
                    let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                    let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                    s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
                }
            }
        }
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_fhiaims() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 8".to_string()).build().unwrap();
        let basis = get_basis("def2-SVP", args);
        let output = write_fhiaims(&basis);
        println!("{output}");
    }
}
