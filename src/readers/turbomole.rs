//! Reader for the TURBOMOLE format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    static ref SECTION_RE: Regex = Regex::new(r"^\$(basis|ecp|cbas|jbas|jkbas)$").unwrap();
    static ref ELEMENT_RE: Regex = Regex::new(r"^([a-zA-Z]{1,3})\s+(.*)$").unwrap();
    static ref SHELL_RE: Regex = Regex::new(r"^(\d+) +([a-zA-Z])$").unwrap();
    static ref ECP_INFO_RE: Regex = RegexBuilder::new(r"^ncore\s*=\s*(\d+)\s+lmax\s*=\s*(\d+)$")
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref ECP_POT_AM_RE: Regex = Regex::new(r"^([a-z])(-[a-z])?$").unwrap();
    static ref EXP_COEF_RE: Regex = Regex::new(&format!(
        r"^(\d+\s+)?({})\s+({})$",
        helpers::FLOATING_RE.as_str(),
        helpers::FLOATING_RE.as_str()
    ))
    .unwrap();
}

/// Parses lines representing all the electron shells for all elements.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    // Strip all lines beginning with $
    let basis_lines = helpers::prune_lines(basis_lines, "$", true, true);

    // Last line should be *
    // We don't need it
    let last = basis_lines.last().map_or(bse_raise!(ValueError, "Empty basis lines"), Ok)?;
    if last != "*" {
        bse_raise!(ValueError, "Missing terminating * line")?;
    }
    let basis_lines = &basis_lines[..basis_lines.len() - 1];

    // Partition based on lines beginning with a character
    let element_blocks =
        helpers::partition_lines(basis_lines, |x| ELEMENT_RE.is_match(x), 1, None, None, None, 4, true)?;

    // Element lines should be surrounded by *
    // Check all first. the partition_lines above will eat part of the previous
    // element if the * is missing
    for element_lines in &element_blocks {
        if element_lines[0] != "*" {
            bse_raise!(ValueError, "Element line not preceded by *")?;
        }
        if element_lines[2] != "*" {
            bse_raise!(ValueError, "Element line not followed by *")?;
        }

        // Check for any other lines starting with *
        for line in element_lines[3..].iter() {
            if line.starts_with('*') {
                bse_raise!(ValueError, "Found line starting with * that probably doesn't belong: {line}")?;
            }
        }
    }

    // Now process them all
    for element_lines in element_blocks {
        let parsed = helpers::parse_line_regex(&ELEMENT_RE, &element_lines[1], "Element line")?;
        let element_sym = &parsed[0];
        let element_Z = lut::element_Z_from_sym(element_sym)
            .map_or(bse_raise!(ValueError, "Unknown element symbol: {element_sym}"), Ok)?;

        // Partition into shells
        let shell_blocks =
            helpers::partition_lines(&element_lines[3..], |x| SHELL_RE.is_match(x), 0, None, None, None, 2, true)?;

        for sh_lines in shell_blocks {
            let parsed = helpers::parse_line_regex(&SHELL_RE, &sh_lines[0], "shell nprim, am")?;
            let nprim = parsed[0].parse().map_or(bse_raise!(ValueError, "Invalid nprim value: {}", parsed[0]), Ok)?;
            let shell_am = lut::amchar_to_int(&parsed[1], false)
                .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", parsed[1]), Ok)?;

            let func_type = lut::function_type_from_am(&shell_am, "gto", "spherical");

            // Check the syntax. There might be an extra ordinal specification
            let mut exponents_and_coefficients = Vec::new();

            for line in &sh_lines[1..] {
                // Check for (expn, coeff) or (iexpn, expn, coeff) format
                let caps = EXP_COEF_RE.captures(line).map_or(
                    bse_raise!(ValueError, "Line does not match format (expn, coeff) or (iexpn, expn, coeff): {line}"),
                    Ok,
                )?;

                // Trim off the optional integer exponent number
                let expn = caps.get(2).unwrap().as_str();
                let coeff = caps.get(3).unwrap().as_str();
                exponents_and_coefficients.push(format!("{expn} {coeff}"));
            }

            let (exponents, coefficients) =
                helpers::parse_primitive_matrix(&exponents_and_coefficients, Some(nprim), Some(1), None)?;

            let shell = BseElectronShell {
                function_type: func_type,
                region: "".to_string(),
                angular_momentum: shell_am,
                exponents,
                coefficients,
            };

            elements.entry(element_Z.to_string()).or_default().electron_shells.get_or_insert_default().push(shell);
        }
    }

    Ok(())
}

/// Parses lines representing all the ECP potentials for a single element.
fn parse_ecp_potential_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    element_lines: &[String],
) -> Result<(), BseError> {
    let parsed = helpers::parse_line_regex(&ELEMENT_RE, &element_lines[0], "Element line")?;
    let element_sym = &parsed[0];
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {element_sym}"), Ok)?;

    // 4th line should be ncore and lmax
    let parsed = helpers::parse_line_regex(&ECP_INFO_RE, &element_lines[1], "ECP ncore, lmax")?;
    let n_elec = parsed[0].parse().map_or(bse_raise!(ValueError, "Invalid ncore value: {}", parsed[0]), Ok)?;
    let max_am: i32 = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid lmax value: {}", parsed[1]), Ok)?;

    // Set the number of electrons
    elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(n_elec);

    // split the remaining lines by lines starting with a character
    let ecp_potentials = helpers::partition_lines(
        &element_lines[2..],
        |x| x.chars().next().is_some_and(|c| c.is_alphabetic()),
        0,
        None,
        None,
        None,
        2,
        true,
    )?;

    // Keep track of what the max AM we actually found is
    let mut found_max = false;
    for pot_lines in ecp_potentials {
        let parsed = helpers::parse_line_regex(&ECP_POT_AM_RE, &pot_lines[0], "ECP potential am")?;
        let pot_am = lut::amchar_to_int(&parsed[0], false)
            .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", parsed[0]), Ok)?;

        let _pot_base_am = if parsed.len() > 1 && !parsed[1].is_empty() {
            let base_am = &parsed[1][1..]; // Strip the - from the beginning
            let base_am = lut::amchar_to_int(base_am, false)
                .map_or(bse_raise!(ValueError, "Unknown base angular momentum: {base_am}"), Ok)?;

            if base_am[0] != max_am {
                bse_raise!(ValueError, "Potential does not use max_am of {max_am}. Uses {}", base_am[0])?;
            }

            Some(base_am)
        } else {
            if found_max {
                bse_raise!(ValueError, "Found multiple potentials with single AM")?;
            }

            if pot_am[0] != max_am {
                bse_raise!(ValueError, "Potential with single AM {} is not the same as lmax = {}", pot_am[0], max_am)?;
            }

            found_max = true;
            None
        };

        let ecp_data = helpers::parse_ecp_table(&pot_lines[1..], &["coeff", "r_exp", "g_exp"], None)?;
        let ecp_pot = BseEcpPotential {
            angular_momentum: pot_am,
            coefficients: ecp_data.coeff,
            ecp_type: "scalar_ecp".to_string(),
            r_exponents: ecp_data.r_exp,
            gaussian_exponents: ecp_data.g_exp,
        };

        elements.entry(element_Z.to_string()).or_default().ecp_potentials.get_or_insert_default().push(ecp_pot);
    }

    Ok(())
}

/// Parses lines representing all the ECP potentials for all elements.
fn parse_ecp_lines(elements: &mut HashMap<String, BseBasisElement>, basis_lines: &[String]) -> Result<(), BseError> {
    // Strip all lines beginning with $
    let basis_lines = helpers::prune_lines(basis_lines, "$", true, true);

    // Last line should be *
    // We don't need it
    let last = basis_lines.last().map_or(bse_raise!(ValueError, "Empty basis lines"), Ok)?;
    if last != "*" {
        bse_raise!(ValueError, "Missing terminating * line")?;
    }
    let basis_lines = &basis_lines[..basis_lines.len() - 1];

    // Partition based on lines beginning with a character
    let element_blocks =
        helpers::partition_lines(basis_lines, |x| ELEMENT_RE.is_match(x), 1, None, None, None, 1, true)?;

    // Element lines should be surrounded by *
    // Check all first. the partition_lines above will eat part of the previous
    // element if the * is missing
    for element_lines in &element_blocks {
        if element_lines[0] != "*" {
            bse_raise!(ValueError, "Element line not preceded by *")?;
        }
        if element_lines[2] != "*" {
            bse_raise!(ValueError, "Element line not followed by *")?;
        }

        // Check for any other lines starting with *
        for line in element_lines[3..].iter() {
            if line.starts_with('*') {
                bse_raise!(ValueError, "Found line starting with * that probably doesn't belong: {line}")?;
            }
        }
    }

    // Now process all elements
    for element_lines in element_blocks {
        // Remove the two * lines and parse using the separate function
        let new_lines = [element_lines[1].clone()].into_iter().chain(element_lines[3..].iter().cloned()).collect_vec();
        parse_ecp_potential_lines(elements, &new_lines)?;
    }

    Ok(())
}

pub fn read_turbomole(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "#", true, true);

    // first line must begin with $, last line must be $end
    if !basis_lines.is_empty() {
        if !basis_lines[0].starts_with('$') {
            bse_raise!(ValueError, "First line does not begin with $. Line: {}", basis_lines[0])?;
        }
        if basis_lines.last().unwrap() != "$end" {
            bse_raise!(ValueError, "Last line of basis is not $end. Line: {}", basis_lines.last().unwrap())?;
        }
    }

    let mut basis_dict = BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "minimal".to_string(), schema_version: "0.1".to_string() },
        elements: HashMap::new(),
        function_types: Vec::new(),
        name: "unknown_basis".to_string(),
        description: "no_description".to_string(),
    };

    // Split into basis and ecp
    // Just split based on lines beginning with $
    let basis_sections = helpers::partition_lines(
        &basis_lines,
        |x| x.starts_with('$') && x != "$end",
        0,
        None,
        Some(1),
        Some(2),
        1,
        true,
    )?;

    for s in basis_sections {
        // Check if section is empty. If so, all lines start with $
        if s.iter().all(|x| x.starts_with('$')) {
            continue;
        }

        if s.is_empty() {
            continue;
        } else if s[0].eq_ignore_ascii_case("$ecp") {
            parse_ecp_lines(&mut basis_dict.elements, &s)?;
        } else if SECTION_RE.is_match(&s[0]) {
            parse_electron_lines(&mut basis_dict.elements, &s)?;
        } else {
            bse_raise!(ValueError, "Unknown section {}", s[0])?;
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
    fn test_read_turbomole() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "turbomole", args);
        let basis = read_turbomole(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
