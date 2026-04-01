//! Reader for the Molcas format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Element line: " H    / inline" or " In.ECP    / inline"
    static ref ELEMENT_LINE_RE: Regex = Regex::new(r"^\s*([A-Za-z]{1,3})(\.ECP)?\s+/.*$").unwrap();
    // Nuclear charge and max_am: "     21.00   3" or "      1.00   1"
    static ref Z_MAX_AM_RE: Regex = Regex::new(
        &format!(r"^\s*(\d+|{})\s+(\d+)$", helpers::FLOATING_RE.as_str())
    ).unwrap();
    // Shell nprim ngen: "     5    3"
    static ref SHELL_NPRIM_NGEN_RE: Regex = Regex::new(r"^\s*(\d+)\s+(\d+)$").unwrap();
    // ECP info: "PP, In, 28, 3 ;"
    static ref ECP_INFO_RE: Regex = Regex::new(r"^PP\s*,\s*([a-zA-Z]+)\s*,\s*(\d+)\s*,\s*(\d+)\s*;$").unwrap();
    // ECP potential begin: "5; !  ul potential"
    static ref ECP_POT_BEGIN_RE: Regex = Regex::new(r"^(\d+)\s*;.*$").unwrap();
}

/// Parses lines representing all the electron shells for a single element.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
    element_Z: &str,
    has_ecp: bool,
) -> Result<(), BseError> {
    if basis_lines.is_empty() {
        return Ok(());
    }

    // First line is nuclear charge and max_am
    let parsed = helpers::parse_line_regex(&Z_MAX_AM_RE, &basis_lines[0], "Electron: Z, max_am")?;
    let nuc_charge_str = &parsed[0];
    let _max_am: i32 = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid max_am: {}", parsed[1]), Ok)?;

    // Parse nuclear charge
    let nuc_charge: f64 = helpers::replace_d(nuc_charge_str)
        .parse()
        .map_or(bse_raise!(ValueError, "Invalid nuclear charge: {}", nuc_charge_str), Ok)?;

    // Is this actually an integer?
    if nuc_charge.fract() != 0.0 {
        bse_raise!(ValueError, "Non-integer specified for nuclear charge: {}", nuc_charge)?;
    }

    let element_Z_int: i32 = element_Z.parse().unwrap();
    let ecp_electrons = element_Z_int - nuc_charge as i32;

    // If the nuclear charge is not equal to the element Z, then this must be an ECP
    if ecp_electrons > 0 {
        elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ecp_electrons);
    } else if has_ecp {
        // ECP electrons should have been set by the ECP parsing
        // Verify consistency
        if let Some(existing) = elements.get(element_Z).and_then(|e| e.ecp_electrons) {
            if existing != ecp_electrons {
                bse_raise!(ValueError, "ECP electrons mismatch: {} vs {}", existing, ecp_electrons)?;
            }
        }
    }

    // Partition into shell blocks - each starts with "* X-type functions" comment
    // followed by nprim ngen line
    let mut shell_am: i32 = 0;
    let mut i = 1;

    while i < basis_lines.len() {
        // Skip comment lines
        if basis_lines[i].starts_with('*') {
            i += 1;
            continue;
        }

        // Check for nprim ngen line
        if SHELL_NPRIM_NGEN_RE.is_match(&basis_lines[i]) {
            let parsed = helpers::parse_line_regex(&SHELL_NPRIM_NGEN_RE, &basis_lines[i], "Shell nprim, ngen")?;
            let nprim: usize = parsed[0].parse().map_or(bse_raise!(ValueError, "Invalid nprim: {}", parsed[0]), Ok)?;
            let ngen: usize = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid ngen: {}", parsed[1]), Ok)?;

            if nprim == 0 {
                bse_raise!(ValueError, "Cannot have 0 primitives in a shell")?;
            }
            if ngen == 0 {
                bse_raise!(ValueError, "Cannot have 0 general contractions in a shell")?;
            }

            i += 1;

            // Read exponents (one per line)
            let exponent_lines = &basis_lines[i..i + nprim];
            let exponents: Vec<String> = exponent_lines
                .iter()
                .map(|line| {
                    let trimmed = line.trim();
                    helpers::replace_d(trimmed.split_whitespace().next().unwrap_or(trimmed))
                })
                .collect();

            i += nprim;

            // Read coefficient matrix (nprim rows, ngen columns)
            let coefficient_lines = &basis_lines[i..i + nprim];
            let coefficients = helpers::parse_matrix(coefficient_lines, Some(nprim), Some(ngen), None)?;

            i += nprim;

            // Transpose coefficients
            let coefficients = misc::transpose_matrix(&coefficients);

            // Now add to the elements
            let func_type = lut::function_type_from_am(&[shell_am], "gto", "spherical");

            let shell = BseElectronShell {
                function_type: func_type,
                region: "".to_string(),
                angular_momentum: vec![shell_am],
                exponents,
                coefficients,
            };

            elements
                .entry(element_Z.to_string())
                .or_default()
                .electron_shells
                .get_or_insert_with(Default::default)
                .push(shell);

            shell_am += 1;
        } else {
            i += 1;
        }
    }

    Ok(())
}

/// Parses lines representing all the ECP potentials for a single element.
fn parse_ecp_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
    element_Z: &str,
) -> Result<(), BseError> {
    if basis_lines.is_empty() {
        return Ok(());
    }

    // Parse the ecp info line
    let parsed = helpers::parse_line_regex(&ECP_INFO_RE, &basis_lines[0], "ECP Info: pp,sym,nelec,max_am")?;
    let element_sym = &parsed[0];
    let ecp_electrons: i32 = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid nelec: {}", parsed[1]), Ok)?;
    let max_am: i32 = parsed[2].parse().map_or(bse_raise!(ValueError, "Invalid max_am: {}", parsed[2]), Ok)?;

    let element_Z_ecp = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Does this block match the element symbol from the main element header?
    if element_Z_ecp.to_string() != element_Z {
        bse_raise!(ValueError, "ECP element Z={} found in block for element Z={}", element_Z_ecp, element_Z)?;
    }

    // Set ECP electrons
    elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ecp_electrons);

    // Now split into potentials
    // The beginning of each potential is a number followed by a semicolon
    let pot_blocks =
        helpers::partition_lines(&basis_lines[1..], |x| ECP_POT_BEGIN_RE.is_match(x), 0, None, None, None, 2, true)?;

    if pot_blocks.len() != (max_am + 1) as usize {
        bse_raise!(ValueError, "Expected {} potentials, but got {}", max_am + 1, pot_blocks.len())?;
    }

    // Set up the AM for the potentials
    let all_pot_am = helpers::potential_am_list(max_am);

    for (idx, pot_lines) in pot_blocks.into_iter().enumerate() {
        let pot_am = all_pot_am[idx];

        let parsed = helpers::parse_line_regex(&ECP_POT_BEGIN_RE, &pot_lines[0], "ECP Potential: # of lines")?;
        let nlines: usize = parsed[0].parse().map_or(bse_raise!(ValueError, "Invalid nlines: {}", parsed[0]), Ok)?;

        if nlines != pot_lines.len() - 1 {
            bse_raise!(ValueError, "Expected {} lines in potential, but got {}", nlines, pot_lines.len() - 1)?;
        }

        // Strip trailing semicolon and split by comma
        let pot_lines: Vec<String> = pot_lines[1..]
            .iter()
            .map(|x| {
                let line = x.trim_end_matches(';');
                line.split(',').map(|s| helpers::replace_d(s.trim())).collect::<Vec<_>>().join(" ")
            })
            .collect();

        // Parse ECP table
        let ecp_data = helpers::parse_ecp_table(&pot_lines, &["r_exp", "g_exp", "coeff"], None)?;

        let ecp_pot = BseEcpPotential {
            angular_momentum: vec![pot_am],
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
    }

    Ok(())
}

pub fn read_molcas(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "*#$", true, true);

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

    // Split into element blocks by "Basis set" lines
    let element_blocks = helpers::partition_lines(
        &basis_lines,
        |x| x == "Basis set",
        0,
        None,
        None,
        None,
        3,
        false, // Don't include the "Basis set" line
    )?;

    for element_lines in element_blocks {
        // Skip empty blocks or "End of basis set" lines
        if element_lines.is_empty() || element_lines[0] == "End of basis set" {
            continue;
        }

        // Find the element line: " H    / inline" or " In.ECP    / inline"
        let mut element_line_idx = None;
        for (idx, line) in element_lines.iter().enumerate() {
            if ELEMENT_LINE_RE.is_match(line) {
                element_line_idx = Some(idx);
                break;
            }
        }

        let element_line_idx = match element_line_idx {
            Some(idx) => idx,
            None => continue, // Skip blocks without element line
        };

        let element_line = &element_lines[element_line_idx];
        let caps = ELEMENT_LINE_RE.captures(element_line).unwrap();
        let element_sym = caps.get(1).unwrap().as_str();
        let element_Z = lut::element_Z_from_sym(element_sym)
            .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

        // Lines after element line
        let remaining_lines: Vec<String> = element_lines[element_line_idx + 1..]
            .iter()
            .filter(|x| x != &"End of basis set" && !x.starts_with("cartesian"))
            .cloned()
            .collect();

        if remaining_lines.is_empty() {
            continue;
        }

        // Check if there's an ECP section (starts with "PP,")
        let ecp_idx = remaining_lines.iter().position(|x| x.starts_with("PP,"));
        let spectral_idx = remaining_lines.iter().position(|x| x == "Spectral");

        if let Some(ecp_idx) = ecp_idx {
            // Has ECP - split into electron and ECP sections
            // Find where ECP ends (before "Spectral" or end)
            let ecp_end = spectral_idx.unwrap_or(remaining_lines.len());

            let electron_lines: Vec<String> = remaining_lines[..ecp_idx].to_vec();
            let ecp_lines: Vec<String> = remaining_lines[ecp_idx..ecp_end].to_vec();

            if !electron_lines.is_empty() {
                parse_electron_lines(&mut basis_dict.elements, &electron_lines, &element_Z.to_string(), true)?;
            }
            if !ecp_lines.is_empty() {
                parse_ecp_lines(&mut basis_dict.elements, &ecp_lines, &element_Z.to_string())?;
            }
        } else {
            // Only electron shells
            parse_electron_lines(&mut basis_dict.elements, &remaining_lines, &element_Z.to_string(), false)?;
        }
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_molcas() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "molcas", args);
        let basis = read_molcas(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_molcas_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "molcas", args);
        let basis = read_molcas(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
