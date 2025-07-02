//! Conversion of basis sets to VeloxChem format.

use crate::prelude::*;

/// Converts a basis set to VeloxChem format
pub fn write_veloxchem(basis: &BseBasis) -> String {
    let mut s = vec![format!("@BASIS_SET {}", basis.name)];

    let mut basis = basis.clone();
    manip::optimize_general(&mut basis);
    manip::uncontract_general(&mut basis);
    manip::uncontract_spdf(&mut basis, 0);
    manip::prune_basis(&mut basis);

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Electron Basis
    if !electron_elements.is_empty() {
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();
            let z_num: i32 = z.parse().unwrap();
            let sym = lut::element_sym_from_Z_with_normalize(z_num).unwrap().to_uppercase();
            let elname = lut::element_name_from_Z(z_num).unwrap().to_uppercase();
            let cont_string = misc::contraction_string(shells, HIJ, INCOMPACT);

            s.push(format!("\n! {elname}       {cont_string}"));
            s.push(format!("@ATOMBASIS {sym}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let ncol = coefficients.len() + 1;
                let nprim = exponents.len();
                let ngen = coefficients.len();

                let am = &shell.angular_momentum;
                // use 'hij' convention, where AM=7 is j
                let amchar = lut::amint_to_char(am, HIJ).to_uppercase();

                s.push(format!("{amchar}    {nprim}    {ngen}"));

                let point_places = (1..=ncol).map(|i| 2 * i + 16 * (i - 1)).collect_vec();
                let exp_coef = [vec![exponents.clone()], coefficients.clone()].concat();
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }

            s.push("@END".to_string());
        }
    }

    let mut joined = s.join("\n") + "\n";
    let md5sum = md5::compute(joined.as_bytes());
    joined.push_str(&format!("{md5sum:?}"));
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_veloxchem() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_veloxchem(&basis);
        println!("{output}");
    }
}
