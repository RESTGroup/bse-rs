//! Conversion of basis sets to cfour/aces2/genbas format

use crate::prelude::*;
use std::cmp::min;

/// Formats an exponent for CFour
fn cfour_exp(e: &str) -> String {
    e.replace('E', "D") + " "
}

/// Formats a coefficient for CFour
fn cfour_coef(c: &str) -> String {
    c.replace('E', "D") + " "
}

/// Formats an exponent for AcesII
fn aces_exp(e: &str) -> String {
    let e_val: f64 = e.parse().unwrap();
    // Some basis sets have negative exponents???
    let mag = if e_val == 0.0 { 0 } else { (e_val.abs().log10() as i32).max(1) };

    // Make room for the negative sign
    let mag = if e_val < 0.0 { mag + 1 } else { mag };

    // Number of decimal places to show
    let ndec = min(7, 14 - 2 - mag) as usize;

    let mut s = format!("{e_val:14.ndec$}");

    // Trim a single trailing zero if there is one
    // and our string takes up all 14 characters
    if !s.starts_with(' ') && s.ends_with('0') {
        s = format!(" {}", s.trim_end_matches('0'));
    }

    s
}

/// Formats a coefficient for AcesII
fn aces_coef(c: &str) -> String {
    let c_val: f64 = c.parse().unwrap();
    format!("{c_val:10.7} ")
}

/// Prints data in columns
fn print_columns(data: &[String], ncol: usize, with_new_line: bool) -> String {
    let mut s = String::new();
    for chunk in data.chunks(ncol) {
        s.push_str(&chunk.join(""));
        if with_new_line {
            s.push('\n');
        }
    }
    s
}

/// Internal function for writing genbas format
fn write_genbas_internal(
    basis: &BseBasis,
    exp_formatter: fn(&str) -> String,
    coef_formatter: fn(&str) -> String,
) -> String {
    // Uncontract all, then make general
    let mut basis = basis.clone();
    manip::make_general(&mut basis, INCOMPACT);
    sort::sort_basis(&mut basis);

    // Elements for which we have electron basis
    let electron_elements =
        basis.elements.iter().filter_map(|(k, v)| v.electron_shells.as_ref().map(|_| k)).sorted().collect_vec();

    // Elements for which we have ECP
    let ecp_elements =
        basis.elements.iter().filter_map(|(k, v)| v.ecp_potentials.as_ref().map(|_| k)).sorted().collect_vec();

    let mut s = Vec::new();
    s.push(String::new()); // Start with empty line

    if !electron_elements.is_empty() {
        // Electron Basis
        for z in electron_elements {
            let data = &basis.elements[z];
            let shells = data.electron_shells.as_ref().unwrap();
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let nshell = shells.len();

            s.push(format!("{}:{}", sym, basis.name));
            s.push(basis.description.clone());
            s.push("".to_string());
            s.push(format!("{nshell:>3}"));

            let mut s_am = String::new();
            let mut s_ngen = String::new();
            let mut s_nprim = String::new();
            for shell in shells {
                s_am.push_str(&format!("{:>5}", shell.angular_momentum[0]));
                s_ngen.push_str(&format!("{:>5}", shell.coefficients.len()));
                s_nprim.push_str(&format!("{:>5}", shell.exponents.len()));
            }

            s.push(s_am);
            s.push(s_ngen);
            s.push(s_nprim);
            s.push("".to_string());

            for shell in shells {
                let exponents: Vec<String> = shell.exponents.iter().map(|x| exp_formatter(x)).collect();
                let coefficients: Vec<Vec<String>> =
                    shell.coefficients.iter().map(|y| y.iter().map(|x| coef_formatter(x)).collect()).collect();

                // Transpose coefficients
                let coefficients_t: Vec<Vec<String>> = (0..coefficients[0].len())
                    .map(|i| coefficients.iter().map(|row| row[i].clone()).collect())
                    .collect();

                s.push(print_columns(&exponents, 5, true));
                for c in coefficients_t {
                    s.push(print_columns(&c, 7, false));
                }
                s.push("".to_string());
            }
        }
    }

    // Write out ECP
    if !ecp_elements.is_empty() {
        s.push("\n".to_string());
        s.push("! Effective core Potentials".to_string());

        for z in ecp_elements {
            let data = &basis.elements[z];
            let sym = lut::element_sym_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();
            let max_ecp_amchar = lut::amint_to_char(&[max_ecp_am], HIK).to_lowercase();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push("*".to_string());
            s.push(format!("{}:{}", sym, basis.name));
            s.push(format!("# {}", basis.description));
            s.push("*".to_string());
            s.push(format!("    NCORE = {}    LMAX = {}", data.ecp_electrons.unwrap(), max_ecp_am));

            for pot in ecp_list {
                let rexponents: Vec<String> = pot.r_exponents.iter().map(|x| x.to_string()).collect();
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK).to_lowercase();

                if am[0] == max_ecp_am {
                    s.push(amchar);
                } else {
                    s.push(format!("{amchar}-{max_ecp_amchar}"));
                }

                let point_places = [6, 18, 25];
                let mut exp_coef = coefficients.clone();
                exp_coef.push(rexponents);
                exp_coef.push(gexponents.clone());
                s.push(printing::write_matrix(&exp_coef, &point_places, SCIFMT_E));
            }
            s.push("*".to_string());
        }
    }

    s.join("\n") + "\n"
}

/// Converts a basis set to CFour format
pub fn write_cfour(basis: &BseBasis) -> String {
    // March 2019
    // Format determined from http://slater.chemie.uni-mainz.de/cfour/index.php?n=Main.NewFormatOfAnEntryInTheGENBASFile
    write_genbas_internal(basis, cfour_exp, cfour_coef)
}

/// Converts a basis set to AcesII format
pub fn write_aces2(basis: &BseBasis) -> String {
    // March 2019
    // Format determined from http://slater.chemie.uni-mainz.de/cfour/index.php?n=Main.OldFormatOfAnEntryInTheGENBASFile
    write_genbas_internal(basis, aces_exp, aces_coef)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_cfour() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_cfour(&basis);
        println!("{output}");
    }

    #[test]
    fn test_write_aces2() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_aces2(&basis);
        println!("{output}");
    }
}
