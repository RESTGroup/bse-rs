//! Conversion of basis sets to Molpro format

use crate::prelude::*;

/// Converts a basis set to Molpro format
pub fn write_molpro(basis: &BseBasis) -> String {
    // Uncontract all, and make as generally-contracted as possible
    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Start out with angular momentum type
    let types = &basis.function_types;
    let harm_type = if types.contains(&"gto_cartesian".to_string()) { "cartesian" } else { "spherical" };
    s.push(harm_type.to_string());

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    if !electron_elements.is_empty() {
        // basis set starts with a string
        s.push("basis={".to_string());

        // Electron Basis
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            s.push("!".to_string());
            s.push(format!(
                "! {:20} {}",
                lut::element_name_from_Z(z.parse().unwrap()).unwrap(),
                misc::contraction_string(shells, HIK, INCOMPACT)
            ));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;

                let am = &shell.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_lowercase();
                s.push(format!("{}, {} , {}", amchar, sym, exponents.join(", ")));

                for c in coefficients {
                    let (first, last) = misc::find_range(c);
                    s.push(format!("c, {}.{}, {}", first + 1, last + 1, c[first..=last].join(", ")));
                }
            }
        }
        s.push("}".to_string());
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n\n! Effective core Potentials".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_lowercase();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("ECP, {}, {}, {} ;", sym, data.ecp_electrons.unwrap(), max_ecp_am));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents;
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_lowercase();

                if am[0] == max_ecp_am {
                    s.push(format!("{}; !  ul potential", rexponents.len()));
                } else {
                    s.push(format!("{}; !  {amchar}-ul potential", rexponents.len()));
                }

                for p in 0..rexponents.len() {
                    s.push(format!("{},{},{};", rexponents[p], gexponents[p], coefficients[0][p]));
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
    fn test_write_molpro() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_molpro(&basis);
        println!("{output}");
    }
}
