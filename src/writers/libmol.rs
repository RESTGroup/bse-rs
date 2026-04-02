//! Writer for Molpro system library format (libmol).

use crate::prelude::*;

/// Find the range in a list of coefficients where the coefficient is nonzero.
fn find_range(coeffs: &[String]) -> (usize, usize) {
    let non_zero: Vec<bool> = coeffs.iter().map(|x| x.parse::<f64>().unwrap() != 0.0).collect();

    let first = non_zero.iter().position(|&x| x).unwrap();
    let last = non_zero.iter().rposition(|&x| x).unwrap();

    (first, last)
}

/// Reshape the input array into blocks of the given size.
fn reshape<T: Clone>(data: &[T], block_size: usize) -> Vec<Vec<T>> {
    let mut output = Vec::new();
    for iblock in 0..data.len().div_ceil(block_size) {
        let start = iblock * block_size;
        let end = std::cmp::min(data.len(), (iblock + 1) * block_size);
        output.push(data[start..end].to_vec());
    }
    output
}

/// Converts a basis set to Molpro system library format.
pub fn write_libmol(basis: &BseBasis) -> String {
    // Uncontract all, and make as generally-contracted as possible
    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
    sort::sort_basis(&mut basis);

    // Start out with angular momentum type
    let types = &basis.function_types;
    let harm_type = if types.contains(&"gto_cartesian".to_string()) { "cartesian" } else { "spherical" };
    let mut s = format!("{harm_type}\n");

    // Elements for which we have electron basis (sorted by Z)
    let electron_elements: Vec<_> = basis
        .elements
        .iter()
        .filter(|(_, v)| v.electron_shells.is_some())
        .map(|(k, _)| k.clone())
        .sorted_by_key(|z| z.parse::<i32>().unwrap_or(0))
        .collect();

    // Elements for which we have ECP (sorted by Z)
    let ecp_elements: Vec<_> = basis
        .elements
        .iter()
        .filter(|(_, v)| v.ecp_potentials.is_some())
        .map(|(k, _)| k.clone())
        .sorted_by_key(|z| z.parse::<i32>().unwrap_or(0))
        .collect();

    if !electron_elements.is_empty() {
        s += "basis={\n";

        for z in &electron_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();

            if let Some(shells) = &data.electron_shells {
                for shell in shells {
                    let exponents = &shell.exponents;
                    let coefficients = &shell.coefficients;
                    let am = &shell.angular_momentum;
                    let amchar = lut::amint_to_char(am, HIK).to_lowercase();

                    let nprim = exponents.len();
                    let ncontr = coefficients.len();

                    // Collect ranges and coefficients
                    let mut ranges = String::new();
                    let mut print_data: Vec<String> = exponents.clone();
                    for c in coefficients {
                        let (first, last) = find_range(c);
                        ranges.push_str(&format!(" {}.{}", first + 1, last + 1));
                        print_data.extend(c[first..=last].iter().cloned());
                    }

                    // Print block entry
                    s.push_str(&format!("{} {} {} : {} {}{}\n", sym, amchar, basis.name, nprim, ncontr, ranges));

                    // Comment
                    let cont_str = data
                        .electron_shells
                        .as_ref()
                        .map_or("".to_string(), |shls| misc::contraction_string(shls, HIK, INCOMPACT));
                    let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap();
                    s.push_str(&format!("{} {} converted by Basis Set Exchange\n", el_name, cont_str));

                    // Output data has 5 entries per row
                    let print_data = reshape(&print_data, 5);
                    for d in print_data {
                        s.push_str(&d.join(" "));
                        s.push('\n');
                    }
                }
            }
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s += "\n\n! Effective core Potentials\n";

        for z in &ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_lowercase();

            if let Some(ecp_potentials) = &data.ecp_potentials {
                let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

                // Sort lowest->highest, then put the highest at the beginning
                let mut ecp_list: Vec<_> = ecp_potentials.iter().collect();
                ecp_list.sort_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum));
                let last = ecp_list.pop().unwrap();
                ecp_list.insert(0, last);

                // Collect data for the printout
                let mut numdata = 0;
                let mut print_blocks: Vec<Vec<String>> = Vec::new();
                for pot in &ecp_list {
                    let rexp = &pot.r_exponents;
                    let gexp = &pot.gaussian_exponents;
                    let coef = &pot.coefficients;

                    let mut block = Vec::new();
                    for i in 0..rexp.len() {
                        block.push(rexp[i].to_string());
                        block.push(gexp[i].clone());
                        block.push(coef[0][i].clone());
                    }
                    print_blocks.push(block);
                    numdata += 3 * rexp.len() + 1;
                }

                // Spin-orbit ECPs are not supported
                let nspinorbit = 0;

                // Print out header
                s.push_str(&format!(
                    "{} ECP : {} {} {} {}\n",
                    sym,
                    data.ecp_electrons.unwrap(),
                    max_ecp_am,
                    nspinorbit,
                    numdata
                ));
                s.push_str(&format!("ECP for {} converted by Basis Set Exchange\n", basis.name));

                for b in &print_blocks {
                    // Print number of terms
                    s.push_str(&format!("{} ", b.len() / 3));

                    // Each line has 6 ECP data
                    let pb = reshape(b, 6);
                    for (d, row) in pb.iter().enumerate() {
                        if d > 0 {
                            s.push_str("  ");
                        }
                        s.push_str(&row.join(" "));
                        s.push('\n');
                    }
                }
            }
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_libmol() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_libmol(&basis);
        println!("{output}");
    }

    #[test]
    fn test_write_libmol_no_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("1-10".to_string()).build().unwrap();
        let basis = get_basis("cc-pVTZ", args);
        let output = write_libmol(&basis);
        println!("{output}");
    }
}
