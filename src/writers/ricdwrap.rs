//! Writer for acCD auxiliary basis generation wrapper (OpenMolcas).

use crate::prelude::*;

/// Converts a basis set to OpenMolcas ricd/acCD input format.
pub fn write_ricdwrap(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
    sort::sort_basis(&mut basis);

    let mut s = String::from(
        "\n&GATEWAY
  ricd
  accd
  cdthreshold=1.0d-4
",
    );

    for (z, data) in basis.elements.iter().sorted_by_key(|(z, _)| z.parse::<i32>().unwrap_or(0)) {
        let has_electron = data.electron_shells.is_some();

        let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        let el_sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();

        s.push_str("Basis set\n");
        let cont_str =
            data.electron_shells.as_ref().map_or("".to_string(), |shls| misc::contraction_string(shls, HIK, INCOMPACT));
        s.push_str(&format!("* {}  {}\n", el_name, cont_str));
        s.push_str(&format!(" {}    / inline\n", el_sym));

        if has_electron {
            let shells = data.electron_shells.as_ref().unwrap();
            let max_am = misc::max_am(shells);

            // Number of electrons
            let nelectrons = z.parse::<i32>().unwrap();

            s.push_str(&format!("{nelectrons:>7}.00   {max_am}\n"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let nprim = exponents.len();
                let ngen = coefficients.len();

                let amchar = lut::amint_to_char(&shell.angular_momentum, HIK).to_uppercase();
                s.push_str(&format!("* {amchar}-type functions\n"));
                s.push_str(&format!("{nprim:>6}    {ngen}\n"));

                s.push_str(&printing::write_matrix(std::slice::from_ref(exponents), &[17], SCIFMT_E));
                s.push('\n');

                let point_places = (1..=ngen).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                s.push_str(&printing::write_matrix(coefficients, &point_places, SCIFMT_E));
                s.push('\n');
            }
        }

        // Make a nucleus; use 10 angstrom distance
        let z_int: i32 = z.parse().unwrap();
        s.push_str(&format!("{} 0.0 0.0 {:.1}\n", el_sym, 10.0 * (z_int - 1) as f64));

        // Are there cartesian shells?
        if has_electron {
            let mut cartesian_shells = HashSet::new();
            for shell in data.electron_shells.as_ref().unwrap() {
                if shell.function_type == "gto_cartesian" {
                    for am in &shell.angular_momentum {
                        cartesian_shells.insert(lut::amint_to_char(&[*am], HIK));
                    }
                }
            }
            if !cartesian_shells.is_empty() {
                s.push_str(&format!("cartesian {}\n", cartesian_shells.iter().join(" ")));
            }
        }

        s.push_str("End of basis set\n\n");
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_ricdwrap() {
        let args = BseGetBasisArgsBuilder::default().elements("1-3".to_string()).build().unwrap();
        let basis = get_basis("cc-pVTZ", args);
        let output = write_ricdwrap(&basis);
        println!("{output}");
    }
}
