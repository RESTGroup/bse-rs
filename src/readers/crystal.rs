//! Reader for the Crystal format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Element block: "Z nshells" or "(Z+200) nshells"
    static ref ELEMENT_BLOCK_RE: Regex = Regex::new(r"^(\d+)\s+(\d+)\s*$").unwrap();
    // Shell descriptor: "ityb lat ng che scal"
    static ref SHELL_DESC_RE: Regex = Regex::new(r"^(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+([0-9.]+)\s*$").unwrap();
    // ECP header: "INPUT"
    static ref ECP_INPUT_RE: Regex = Regex::new(r"^INPUT\s*$").unwrap();
    // ECP descriptor: "Zeff m num_s num_p num_d num_f num_g"
    static ref ECP_DESC_RE: Regex = Regex::new(r"^(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$").unwrap();
    // End marker: "99 0"
    static ref END_RE: Regex = Regex::new(r"^99\s+0\s*$").unwrap();
}

/// Convert Crystal lat code to angular momentum
fn lat_to_am(lat: i32) -> Vec<i32> {
    match lat {
        0 => vec![0],     // S shell
        1 => vec![0, 1],  // SP shell
        n => vec![n - 1], // P, D, F, G, H shells (lat = am + 1)
    }
}

/// Parses lines representing all the electron shells for a single element.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    if basis_lines.is_empty() {
        return Ok(());
    }

    // First line: element Z and number of shells
    let parsed = helpers::parse_line_regex(&ELEMENT_BLOCK_RE, &basis_lines[0], "Element block")?;
    let nat: i32 = parsed[0].parse().unwrap();
    let nshells: usize = parsed[1].parse().unwrap();

    // Determine actual Z (subtract 200 if ECP element)
    let (element_Z, has_ecp) = if nat >= 200 { (nat - 200, true) } else { (nat, false) };

    let mut iline = 1;

    // If has ECP, parse ECP section first
    if has_ecp {
        // Look for INPUT marker
        while iline < basis_lines.len() && !ECP_INPUT_RE.is_match(&basis_lines[iline]) {
            iline += 1;
        }

        if iline < basis_lines.len() {
            iline += 1; // Skip INPUT line

            // Parse ECP descriptor
            let parsed = helpers::parse_line_regex(&ECP_DESC_RE, &basis_lines[iline], "ECP descriptor")?;
            let zeff: i32 = parsed[0].parse().unwrap();
            let _m: i32 = parsed[1].parse().unwrap(); // Scalar terms (not used)
            let num_terms: [usize; 5] = [
                parsed[2].parse().unwrap(),
                parsed[3].parse().unwrap(),
                parsed[4].parse().unwrap(),
                parsed[5].parse().unwrap(),
                parsed[6].parse().unwrap(),
            ];

            // ECP electrons = Z - Zeff
            let ecp_electrons: i32 = element_Z - zeff;
            elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ecp_electrons);

            iline += 1;

            // Parse ECP entries for each AM
            for am in 0..5 {
                if num_terms[am] == 0 {
                    continue;
                }

                let mut g_exp = Vec::new();
                let mut coeff = Vec::new();
                let mut r_exp = Vec::new();

                for _ in 0..num_terms[am] {
                    let parts: Vec<&str> = basis_lines[iline].split_whitespace().collect();
                    if parts.len() != 3 {
                        bse_raise!(ValueError, "Invalid ECP entry: {}", basis_lines[iline])?;
                    }
                    g_exp.push(helpers::replace_d(parts[0]));
                    coeff.push(helpers::replace_d(parts[1]));
                    r_exp.push(
                        parts[2].parse().map_err(|_| BseError::ValueError(format!("Invalid r_exp: {}", parts[2])))?,
                    );
                    iline += 1;
                }

                let ecp_pot = BseEcpPotential {
                    angular_momentum: vec![am as i32],
                    coefficients: vec![coeff],
                    ecp_type: "scalar_ecp".to_string(),
                    r_exponents: r_exp,
                    gaussian_exponents: g_exp,
                };

                elements
                    .entry(element_Z.to_string())
                    .or_default()
                    .ecp_potentials
                    .get_or_insert_with(Default::default)
                    .push(ecp_pot);
            }
        }
    }

    // Parse electron shells
    // The crystal writer uncontracts all shells, so each shell has 1 contraction
    for _ in 0..nshells {
        if iline >= basis_lines.len() {
            break;
        }

        // Skip ECP marker lines if present
        if ECP_INPUT_RE.is_match(&basis_lines[iline]) {
            // This shouldn't happen for non-ECP elements, but handle it
            iline += 1;
            continue;
        }

        let parsed = helpers::parse_line_regex(&SHELL_DESC_RE, &basis_lines[iline], "Shell descriptor")?;
        let _ityb: i32 = parsed[0].parse().unwrap();
        let lat: i32 = parsed[1].parse().unwrap();
        let nprim: usize = parsed[2].parse().unwrap();
        let _che: i32 = parsed[3].parse().unwrap();
        let _scal: f64 = parsed[4].parse().unwrap();

        let shell_am = lat_to_am(lat);
        let ncont = if lat == 1 { 2 } else { 1 }; // SP shells have 2 contractions

        iline += 1;

        // Read matrix: nprim rows, (1 + ncont) columns
        let matrix_data =
            helpers::parse_matrix(&basis_lines[iline..iline + nprim], Some(nprim), Some(1 + ncont), None)?;

        let exponents: Vec<String> = matrix_data.iter().map(|row| helpers::replace_d(&row[0])).collect();
        let coefficients: Vec<Vec<String>> = if lat == 1 {
            // SP shell: two coefficients per row
            vec![
                matrix_data.iter().map(|row| helpers::replace_d(&row[1])).collect(),
                matrix_data.iter().map(|row| helpers::replace_d(&row[2])).collect(),
            ]
        } else {
            // Single contraction
            vec![matrix_data.iter().map(|row| helpers::replace_d(&row[1])).collect()]
        };

        iline += nprim;

        let func_type = lut::function_type_from_am(&shell_am, "gto", "spherical");

        let shell = BseElectronShell {
            function_type: func_type,
            region: "".to_string(),
            angular_momentum: shell_am,
            exponents,
            coefficients,
        };

        elements
            .entry(element_Z.to_string())
            .or_default()
            .electron_shells
            .get_or_insert_with(Default::default)
            .push(shell);
    }

    Ok(())
}

pub fn read_crystal(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    // Crystal format uses '!' for comments in header, but actual basis data has no
    // comments Remove comment lines and blank lines from header
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "!#", true, true);

    let mut basis_dict = BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "minimal".to_string(), schema_version: "0.1".to_string() },
        elements: HashMap::new(),
        function_types: Vec::new(),
        name: "unknown_basis".to_string(),
        description: "no_description".to_string(),
    };

    if basis_lines.is_empty() {
        return Ok(basis_dict);
    }

    // Partition into element blocks - blocks start with element Z
    // The last block ends with "99 0"
    let element_blocks = helpers::partition_lines(
        &basis_lines,
        |x| ELEMENT_BLOCK_RE.is_match(x) || END_RE.is_match(x),
        0,
        None,
        None,
        None,
        1, // Minimum: just need element header (some blocks might have shells that get parsed later)
        true,
    )?;

    for element_lines in element_blocks {
        // Skip blocks that end with the terminator or have only the terminator
        if element_lines.iter().any(|x| END_RE.is_match(x)) {
            continue;
        }
        // Skip blocks that don't have at least an element header
        if element_lines.len() < 2 {
            continue;
        }
        parse_electron_lines(&mut basis_dict.elements, &element_lines)?;
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_crystal() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 6-O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-SVP", "crystal", args);
        let basis = read_crystal(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_crystal_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "crystal", args);
        let basis = read_crystal(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
