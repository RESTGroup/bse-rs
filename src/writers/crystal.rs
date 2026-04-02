//! Conversion of basis sets to Crystal format.

use crate::prelude::*;

/// Converts a basis set to Crystal format
pub fn write_crystal(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

    // Elements for which we have electron basis
    let electron_elements: Vec<_> =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).collect();

    // Elements for which we have ECP
    let ecp_elements: Vec<_> =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).collect();

    let mut s = Vec::new();

    // Basis sets written together
    for (z, data) in basis.elements.iter().sorted_by_key(|(k, _)| k.parse::<i32>().unwrap()) {
        let nat: i32 = match z.parse() {
            Ok(n) if n >= 99 => continue, // Skip elements beyond Z=98
            Ok(n) if ecp_elements.contains(&z) => n + 200,
            Ok(n) => n,
            Err(_) => continue,
        };

        // First line: nuclear charge and number of shells
        if let Some(shells) = &data.electron_shells {
            s.push(format!("{nat} {}", shells.len()));
        } else {
            continue;
        }

        // Handle ECP if present
        if ecp_elements.contains(&z) {
            let ecp_electrons = data.ecp_electrons.unwrap();
            let Zeff = z.parse::<i32>().unwrap() - ecp_electrons;

            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            if max_ecp_am > 4 {
                panic!("ECP contains l={max_ecp_am} term but Crystal format only supports up to g projectors!");
            }

            let mut ecp_entries = Vec::new();
            let mut num_terms = Vec::new();

            // Helper to add terms for a given angular momentum
            let add_term = |am: i32| {
                let mut n_terms = 0;
                let mut entry = String::new();
                for term in ecp_potentials.iter().filter(|k| k.angular_momentum[0] == am) {
                    let exps = &term.gaussian_exponents;
                    let coefs = &term.coefficients[0];
                    let rexp = &term.r_exponents;

                    for i in 0..exps.len() {
                        // r exponent in CRYSTAL format is the full one: it does NOT have the r^{-2}
                        // prefactor as in other formats
                        entry.push_str(&format!("{} {} {}\n", exps[i], coefs[i], rexp[i] - 2));
                        n_terms += 1;
                    }
                }
                (entry, n_terms)
            };

            // The highest-am projector comes first
            let (entry, n) = add_term(max_ecp_am);
            ecp_entries.push(entry);
            num_terms.push(n);

            // Then lower angular momenta
            for am in 0..max_ecp_am {
                let (entry, n) = add_term(am);
                ecp_entries.push(entry);
                num_terms.push(n);
            }

            // Pad with zeros for remaining angular momenta (up to 4 total: s, p, d, f, g)
            for _ in max_ecp_am..5 {
                num_terms.push(0);
            }

            // Print ECP header
            s.push("INPUT".to_string());
            let header = format!("{Zeff}").to_string() + &num_terms.iter().map(|t| format!(" {t}")).collect::<String>();
            s.push(header);
            // Add ECP data
            for entry in ecp_entries {
                s.push(entry.trim().to_string());
            }
        }

        // Handle basis functions
        if electron_elements.contains(&z) {
            let shells = data.electron_shells.as_ref().unwrap();
            for shell in shells {
                let am = &shell.angular_momentum;
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;

                // Determine shell type
                let (ityb, lat) = match am.as_slice() {
                    [0, 1] => (0, 1),         // SP shell
                    [a] if *a == 0 => (0, 0), // S shell
                    [a] => (0, a + 1),        // P, D, etc.
                    _ => panic!("Crystal interface does not handle other combined shells than SP shells"),
                };

                let ng = exponents.len();
                let ncol = coefficients.len() + 1;
                let che = 0; // Formal charge (unknown)
                let scal = 1.0; // Scale factor

                // Print shell descriptor
                s.push(format!("{ityb} {lat} {ng} {che} {scal:.1}"));

                // Print out contractions
                let point_places = (1..=ncol).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_D));
            }
        }
    }

    // End of basis set input
    s.push("99 0".to_string());
    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_crystal() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 8, 17".to_string()).build().unwrap();
        let basis = get_basis("def2-SVP", args);
        let output = write_crystal(&basis);
        println!("{output}");
    }
}
