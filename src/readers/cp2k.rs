//! Reader for the CP2K format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Element block: "SYMBOL basis_name"
    static ref ELEMENT_BLOCK_RE: Regex = Regex::new(r"^([A-Za-z]{1,3})\s+(\S+)$").unwrap();
    // Shell count: just a number (after trimming)
    static ref SHELL_COUNT_RE: Regex = Regex::new(r"^\d+$").unwrap();
    // Shell descriptor: "1 min_am max_am nprim ncont..."
    static ref SHELL_DESC_RE: Regex = Regex::new(r"^1\s+(\d+)\s+(\d+)\s+(\d+)\s+(.+)$").unwrap();
    // ECP header: "## Effective core potentials" (becomes empty line after comment removal)
    // ECP name: "basis_name_ECP"
    static ref ECP_NAME_RE: Regex = Regex::new(r"^(\S+)$").unwrap();
    // ECP element: "SYM nelec N"
    static ref ECP_ELEMENT_RE: Regex = Regex::new(r"^([A-Za-z]{1,3})\s+nelec\s+(\d+)\s*$").unwrap();
    // ECP potential: "SYM ul" or "SYM AM"
    static ref ECP_POTENTIAL_RE: Regex = Regex::new(r"^([A-Za-z]{1,3})\s+([A-Za-z]+)\s*$").unwrap();
    // ECP end: "END basis_name"
    static ref ECP_END_RE: Regex = Regex::new(r"^END\s+(\S+)$").unwrap();
}

/// Parses lines representing all the electron shells for a single element.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    // Skip empty blocks
    if basis_lines.is_empty() {
        return Ok(());
    }

    // Skip non-element blocks (look for lines that match element pattern)
    let mut start_idx = 0;
    for (idx, line) in basis_lines.iter().enumerate() {
        if ELEMENT_BLOCK_RE.is_match(line) {
            start_idx = idx;
            break;
        }
        // If we don't find an element line, skip this block
        if idx == basis_lines.len() - 1 {
            return Ok(());
        }
    }

    let basis_lines = &basis_lines[start_idx..];
    if basis_lines.is_empty() {
        return Ok(());
    }

    // Line 0: element symbol and basis name
    let parsed = helpers::parse_line_regex(&ELEMENT_BLOCK_RE, &basis_lines[0], "Element line")?;
    let element_sym = &parsed[0];
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Line 1: number of shells
    if basis_lines.len() < 2 {
        return Ok(());
    }
    if !SHELL_COUNT_RE.is_match(&basis_lines[1]) {
        return Ok(()); // Skip if no valid shell count
    }
    let nshells: usize = basis_lines[1].parse().unwrap();

    let mut iline = 2;

    for _ in 0..nshells {
        // Shell descriptor: "1 min_am max_am nprim ncont..."
        let parsed = helpers::parse_line_regex(&SHELL_DESC_RE, &basis_lines[iline], "Shell descriptor")?;
        let min_am: i32 = parsed[0].parse().unwrap();
        let max_am: i32 = parsed[1].parse().unwrap();
        let nprim: usize = parsed[2].parse().unwrap();
        let ncont_str = &parsed[3];

        // Parse number of contractions
        let ncont_values: Vec<i32> = ncont_str.split_whitespace().map(|s| s.parse().unwrap()).collect();

        // Determine angular momentum
        let shell_am: Vec<i32> = (min_am..=max_am).collect();

        // Number of contractions is sum of all ncont values for combined shells
        // or the single value for regular shells
        let ncont = ncont_values.iter().sum::<i32>() as usize;

        iline += 1;

        // Read matrix: nprim rows, (1 + ncont) columns (exponents + coefficients)
        let matrix_data =
            helpers::parse_matrix(&basis_lines[iline..iline + nprim], Some(nprim), Some(1 + ncont), None)?;

        let exponents: Vec<String> = matrix_data.iter().map(|row| helpers::replace_d(&row[0])).collect();
        let coefficients: Vec<Vec<String>> =
            (1..=ncont).map(|i| matrix_data.iter().map(|row| helpers::replace_d(&row[i])).collect()).collect();

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

/// Parses lines representing all the ECP potentials for a single element.
fn parse_ecp_lines(elements: &mut HashMap<String, BseBasisElement>, basis_lines: &[String]) -> Result<(), BseError> {
    let mut iline = 0;

    while iline < basis_lines.len() {
        if ECP_ELEMENT_RE.is_match(&basis_lines[iline]) {
            let parsed = helpers::parse_line_regex(&ECP_ELEMENT_RE, &basis_lines[iline], "ECP element")?;
            let element_sym = &parsed[0];
            let ecp_electrons: i32 = parsed[1].parse().unwrap();

            let element_Z = lut::element_Z_from_sym(element_sym)
                .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

            elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ecp_electrons);

            iline += 1;

            // Parse potentials until we reach another element or END
            while iline < basis_lines.len()
                && !ECP_ELEMENT_RE.is_match(&basis_lines[iline])
                && !ECP_END_RE.is_match(&basis_lines[iline])
            {
                if ECP_POTENTIAL_RE.is_match(&basis_lines[iline]) {
                    let parsed = helpers::parse_line_regex(&ECP_POTENTIAL_RE, &basis_lines[iline], "ECP potential")?;
                    let am_char = &parsed[1];

                    // "ul" means the highest AM potential
                    // For now, parse as the character given
                    let pot_am = if am_char.to_lowercase() == "ul" {
                        // We need to figure out what AM this is from context
                        // In CP2K format, ul is always the highest AM potential
                        // We'll handle this after parsing all potentials
                        vec![-1] // placeholder, will be fixed later
                    } else {
                        lut::amchar_to_int(am_char, true)
                            .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?
                    };

                    iline += 1;

                    // Parse ECP data lines until we hit another potential or element
                    let mut ecp_data_lines = Vec::new();
                    while iline < basis_lines.len()
                        && !ECP_POTENTIAL_RE.is_match(&basis_lines[iline])
                        && !ECP_ELEMENT_RE.is_match(&basis_lines[iline])
                        && !ECP_END_RE.is_match(&basis_lines[iline])
                    {
                        ecp_data_lines.push(basis_lines[iline].clone());
                        iline += 1;
                    }

                    // Parse ECP table: r_exp, g_exp, coefficients
                    let ecp_data = helpers::parse_ecp_table(&ecp_data_lines, &["r_exp", "g_exp", "coeff"], None)?;

                    let ecp_pot = BseEcpPotential {
                        angular_momentum: pot_am,
                        coefficients: ecp_data.coeff,
                        ecp_type: "scalar_ecp".to_string(),
                        r_exponents: ecp_data.r_exp,
                        gaussian_exponents: ecp_data.g_exp,
                    };

                    elements
                        .entry(element_Z.to_string())
                        .or_default()
                        .ecp_potentials
                        .get_or_insert_with(Default::default)
                        .push(ecp_pot);
                } else {
                    iline += 1;
                }
            }

            // Fix ul potentials: set them to the max AM found
            let ecp_potentials = elements.get(&element_Z.to_string()).and_then(|e| e.ecp_potentials.as_ref());

            if let Some(potentials) = ecp_potentials {
                let max_am = potentials
                    .iter()
                    .filter(|p| p.angular_momentum[0] >= 0)
                    .map(|p| p.angular_momentum[0])
                    .max()
                    .unwrap_or(0);

                // Update ul potentials to have max_am
                if let Some(element) = elements.get_mut(&element_Z.to_string()) {
                    if let Some(potentials) = &mut element.ecp_potentials {
                        for pot in potentials.iter_mut() {
                            if pot.angular_momentum[0] == -1 {
                                pot.angular_momentum = vec![max_am];
                            }
                        }
                    }
                }
            }
        } else if ECP_END_RE.is_match(&basis_lines[iline]) {
            // End of ECP section
            break;
        } else {
            iline += 1;
        }
    }

    Ok(())
}

pub fn read_cp2k(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    // Remove comments but keep other lines
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "#", true, true);

    let mut basis_dict = BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "minimal".to_string(), schema_version: "0.1".to_string() },
        elements: HashMap::new(),
        function_types: Vec::new(),
        name: "unknown_basis".to_string(),
        description: "no_description".to_string(),
    };

    // Empty file?
    if basis_lines.is_empty() {
        return Ok(basis_dict);
    }

    // Find ECP section start - look for line ending with "_ECP"
    let ecp_start_idx = basis_lines.iter().position(|x| x.ends_with("_ECP"));

    // Split into electron and ECP sections
    let electron_lines = if let Some(idx) = ecp_start_idx { &basis_lines[..idx] } else { &basis_lines };

    let ecp_lines = if let Some(idx) = ecp_start_idx {
        // Skip ECP name line
        &basis_lines[idx + 1..]
    } else {
        &[]
    };

    // Partition electron section into element blocks
    let element_blocks = helpers::partition_lines(
        electron_lines,
        |x| ELEMENT_BLOCK_RE.is_match(x),
        0,
        None,
        None,
        None,
        3, // Minimum size: element line + shell count + at least one shell
        true,
    )?;

    for element_lines in element_blocks {
        parse_electron_lines(&mut basis_dict.elements, &element_lines)?;
    }

    // Parse ECP section if present
    if !ecp_lines.is_empty() {
        parse_ecp_lines(&mut basis_dict.elements, ecp_lines)?;
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_cp2k() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 6-O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVTZ", "cp2k", args);
        let basis = read_cp2k(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_cp2k_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "cp2k", args);
        let basis = read_cp2k(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
