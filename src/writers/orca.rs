//! Conversion of basis sets to ORCA format.

use crate::prelude::*;

/// Writes the ECP basis part for ORCA format
fn write_orca_ecp_basis(basis: &BseBasis, ecp_elements: &[&String]) -> String {
    let mut s = vec![];
    s.push("".to_string());

    for z in ecp_elements {
        s.push("".to_string());

        let data = &basis.elements[*z];
        let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
        let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
        let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIJ);

        // Sort lowest->highest
        let ecp_list =
            ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();

        // Could probably be basis.names[0]-ECP, but seems like special characters
        // would cause problems
        let ecp_name = "NewECP";
        s.push(format!("{ecp_name} {sym}"));
        s.push(format!("  N_core {}", data.ecp_electrons.unwrap()));
        s.push(format!("  lmax {max_ecp_amchar}"));

        for pot in ecp_list {
            let rexponents = &pot.r_exponents.iter().map(|x| x.to_string()).collect_vec();
            let gexponents = &pot.gaussian_exponents;
            let coefficients = &pot.coefficients;
            let nprim = rexponents.len();

            let am = &pot.angular_momentum;
            let amchar = lut::amint_to_char(am, HIJ);

            // Title line
            s.push(format!("  {amchar} {nprim}"));

            // Include an index column
            let idx_column = (1..=nprim).map(|x| x.to_string()).collect_vec();
            let point_places = [4, 12, 27, 36];
            let mut matrix_data = vec![idx_column, gexponents.clone()];
            matrix_data.extend(coefficients.clone());
            matrix_data.push(rexponents.clone());
            s.push(printing::write_matrix(&matrix_data, &point_places, SCIFMT_E));
        }

        s.push("end".to_string());
    }

    s.join("\n")
}

/// Converts a basis set to ORCA format
pub fn write_orca(basis: &BseBasis) -> String {
    writers::gamess_us::write_gamess_us_common(basis, write_orca_ecp_basis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_orca() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_orca(&basis);
        println!("{output}");
    }
}
