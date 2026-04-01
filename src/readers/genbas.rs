//! Reader for the Genbas/CFOUR/ACESII format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Element block: "H:basis_name" or "IN:basis_name"
    static ref ELEMENT_BLOCK_RE: Regex = Regex::new(r"^([a-zA-Z]{1,3}):(.*)$").unwrap();
    // ECP block: "ncore = X lmax = Y"
    static ref ECP_BLOCK_RE: Regex = RegexBuilder::new(r"^ncore\s*=\s*(\d+)\s+lmax\s*=\s*(\d+)\s*$")
        .case_insensitive(true)
        .build()
        .unwrap();
}

/// Parses lines representing all the electron shells for a single element.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    // Line 0: element, basis name
    // Line 1: comment
    // Line 2: blank
    // Line 3: nshell

    let parsed = helpers::parse_line_regex(&ELEMENT_BLOCK_RE, &basis_lines[0], "Element line")?;
    let element_sym = &parsed[0];
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Skip to line after the blank (line 3 has nshell)
    // Line 2 should be blank
    let mut iline = 2;

    // Remove expected blank line
    if !basis_lines[iline].is_empty() {
        bse_raise!(ValueError, "Expected blank line, found: {}", basis_lines[iline])?;
    }
    iline += 1;

    // Read nshell
    let nshell: usize = basis_lines[iline]
        .trim()
        .parse()
        .map_or(bse_raise!(ValueError, "Invalid nshell: {}", basis_lines[iline]), Ok)?;
    iline += 1;

    if nshell == 0 {
        return Ok(());
    }

    // Read AM for each shell (on one line)
    let shell_ams: Vec<i32> = basis_lines[iline]
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| BseError::ValueError(format!("Invalid AM: {}", s))))
        .collect::<Result<Vec<_>, _>>()?;
    iline += 1;

    // Read ngen for each shell (on one line)
    let shell_ngens: Vec<usize> = basis_lines[iline]
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| BseError::ValueError(format!("Invalid ngen: {}", s))))
        .collect::<Result<Vec<_>, _>>()?;
    iline += 1;

    // Read nprim for each shell (on one line)
    let shell_nprims: Vec<usize> = basis_lines[iline]
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| BseError::ValueError(format!("Invalid nprim: {}", s))))
        .collect::<Result<Vec<_>, _>>()?;
    iline += 1;

    if shell_ams.len() != nshell || shell_ngens.len() != nshell || shell_nprims.len() != nshell {
        bse_raise!(
            ValueError,
            "Inconsistent shell data: expected {} shells, got AM={}, ngen={}, nprim={}",
            nshell,
            shell_ams.len(),
            shell_ngens.len(),
            shell_nprims.len()
        )?;
    }

    // Loop over all shells
    for shell_idx in 0..nshell {
        let shell_am = vec![shell_ams[shell_idx]];
        let nprim = shell_nprims[shell_idx];
        let ngen = shell_ngens[shell_idx];

        let func_type = lut::function_type_from_am(&shell_am, "gto", "spherical");

        // Remove blank line before exponents
        if iline < basis_lines.len() && basis_lines[iline].is_empty() {
            iline += 1;
        }

        // Read exponents - all on one or more lines, followed by a blank line
        // Count how many lines of exponents we need
        let mut exp_lines = 0;
        for i in iline..basis_lines.len() {
            if basis_lines[i].is_empty() {
                break;
            }
            exp_lines += 1;
        }

        // Parse all exponents
        let exponents: Vec<String> = basis_lines[iline..iline + exp_lines]
            .iter()
            .flat_map(|line| line.split_whitespace().map(helpers::replace_d).collect::<Vec<_>>())
            .collect();
        iline += exp_lines;

        if exponents.len() != nprim {
            bse_raise!(ValueError, "Expected {} exponents, found {}", nprim, exponents.len())?;
        }

        // Remove blank line between exponents and coefficients
        if iline < basis_lines.len() && basis_lines[iline].is_empty() {
            iline += 1;
        }

        // Read coefficient matrix - nprim rows, ngen columns per row
        // Coefficients may span multiple lines per row
        let mut coeff_matrix: Vec<Vec<String>> = Vec::new();
        for _ in 0..nprim {
            let mut row: Vec<String> = Vec::new();
            while row.len() < ngen && iline < basis_lines.len() {
                let nums: Vec<String> = basis_lines[iline].split_whitespace().map(helpers::replace_d).collect();
                row.extend(nums);
                iline += 1;
            }
            if row.len() != ngen {
                bse_raise!(ValueError, "Expected {} coefficients in row, found {}", ngen, row.len())?;
            }
            coeff_matrix.push(row);
        }

        // Transpose coefficients
        let coefficients = misc::transpose_matrix(&coeff_matrix);

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
    // First line should be "ELEMENT:name"
    let parsed = helpers::parse_line_regex(&ELEMENT_BLOCK_RE, &basis_lines[0], "Element line")?;
    let element_sym = &parsed[0];
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Skip comment lines (starting with #) and find ncore/lmax line
    let mut ncore = 0;
    let mut lmax = 0;
    let mut ncore_line_idx = None;

    for (idx, line) in basis_lines.iter().enumerate().skip(1) {
        if let Some(caps) = ECP_BLOCK_RE.captures(line) {
            ncore = caps.get(1).unwrap().as_str().parse().unwrap();
            lmax = caps.get(2).unwrap().as_str().parse().unwrap();
            ncore_line_idx = Some(idx);
            break;
        }
    }

    let ncore_line_idx = match ncore_line_idx {
        Some(idx) => idx,
        None => bse_raise!(ValueError, "No ncore/lmax line found in ECP block")?,
    };

    elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ncore);

    // Parse the potentials - format after ncore/lmax: potential blocks starting
    // with AM char
    let ecp_lines = &basis_lines[ncore_line_idx + 1..];

    // Remove terminating * if present
    let ecp_lines: Vec<String> = ecp_lines.iter().filter(|x| *x != "*").cloned().collect();

    // Partition into potential blocks
    let ecp_potentials = helpers::partition_lines(
        &ecp_lines,
        |x| x.chars().next().is_some_and(|c| c.is_alphabetic()),
        0,
        None,
        None,
        None,
        2,
        true,
    )?;

    // Keep track of what the max AM we actually found is
    let mut _found_max = false;
    for pot_lines in ecp_potentials {
        // Parse potential AM: "s-ul" or "s" or "ul" or "s-f" format
        let first_line = &pot_lines[0];
        let parts: Vec<&str> = first_line.split('-').collect();

        let pot_am = if parts.len() == 1 {
            // Single AM like "f" - this is the ul potential
            let am_char = parts[0].trim();
            lut::amchar_to_int(am_char, false)
                .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?
        } else {
            // Format like "s-f" - first is the AM, second is the max AM
            let am_char = parts[0].trim();
            lut::amchar_to_int(am_char, false)
                .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?
        };

        if pot_am[0] == lmax {
            _found_max = true;
        }

        // Parse ECP table - format is coeff, r_exp, g_exp
        let ecp_data = helpers::parse_ecp_table(&pot_lines[1..], &["coeff", "r_exp", "g_exp"], None)?;

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
    }

    Ok(())
}

pub fn read_genbas(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    // We leave in blank lines - they are significant
    // Leave strip_end_blanks to True though
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "!#", false, true);

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

    // Split into element blocks
    // For genbas format, blocks start with "ELEMENT:name" or "*ELEMENT:name"
    let element_blocks = helpers::partition_lines(
        &basis_lines,
        |x| {
            // Match "H:name" or "IN:name" patterns (element blocks)
            if ELEMENT_BLOCK_RE.is_match(x) {
                return true;
            }
            // Check if line starts with * followed by element block
            if x.starts_with('*') && x.len() > 1 {
                let rest = &x[1..];
                return ELEMENT_BLOCK_RE.is_match(rest);
            }
            false
        },
        0,
        Some(1),
        None,
        None,
        1, // Lower min_size to allow smaller blocks
        true,
    )?;

    for element_lines in element_blocks {
        // Skip empty blocks
        if element_lines.is_empty() {
            continue;
        }

        // Find the element block line and skip leading * lines
        let mut start_idx = 0;
        let mut is_ecp = false;

        for (idx, line) in element_lines.iter().enumerate() {
            if ELEMENT_BLOCK_RE.is_match(line) {
                start_idx = idx;
                break;
            }
            if line.starts_with('*') && line.len() > 1 && ELEMENT_BLOCK_RE.is_match(&line[1..]) {
                start_idx = idx;
                is_ecp = true;
                break;
            }
            // Handle case where * is on its own line
            if line == "*" {
                is_ecp = true;
                start_idx = idx + 1;
            }
        }

        let element_lines: Vec<String> = element_lines[start_idx..].to_vec();

        if element_lines.is_empty() {
            continue;
        }

        // Fix the first line if it starts with *
        let element_lines: Vec<String> = if element_lines[0].starts_with('*') && element_lines[0].len() > 1 {
            let mut lines = element_lines.clone();
            lines[0] = lines[0][1..].to_string();
            lines
        } else {
            element_lines
        };

        if element_lines.is_empty() {
            continue;
        }

        // Check if this is an ECP block
        let has_ecp_info = element_lines.iter().any(|l| ECP_BLOCK_RE.is_match(l));
        if has_ecp_info || is_ecp {
            parse_ecp_lines(&mut basis_dict.elements, &element_lines)?;
        } else {
            parse_electron_lines(&mut basis_dict.elements, &element_lines)?;
        }
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

/// Read CFOUR format (alias for genbas)
pub fn read_cfour(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    read_genbas(basis_str)
}

/// Read ACESII format (alias for genbas)
pub fn read_aces2(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    read_genbas(basis_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_genbas() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "cfour", args);
        let basis = read_genbas(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_genbas_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "cfour", args);
        let basis = read_genbas(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
