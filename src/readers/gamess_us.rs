//! Reader for the GAMESS US format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Element block: element name on its own line
    static ref ELEMENT_BLOCK_RE: Regex = Regex::new(r"^\s*([A-Za-z]+)\s*$").unwrap();
    // Shell block: "AM nprim" e.g., "S   3"
    static ref SHELL_BLOCK_RE: Regex = Regex::new(r"^\s*([SPDFGHIKLMN])\s+(\d+)\s*$").unwrap();
    // Contraction line: "index exponent coefficient"
    static ref CONTRACTION_RE: Regex = Regex::new(
        &format!(r"^\s*(\d+)\s+({})\s+({})\s*$", helpers::FLOATING_RE.as_str(), helpers::FLOATING_RE.as_str())
    ).unwrap();
    // ECP block: "ELEMENT-ECP GEN nelec lmax"
    static ref ECP_BLOCK_RE: Regex = Regex::new(r"^\s*([A-Za-z]+)-ECP\s+GEN\s+(\d+)\s+(\d+)\s*$").unwrap();
    // ECP shell: "nlines ----- am-ul potential -----"
    static ref ECP_SHELL_RE: Regex = Regex::new(r"^\s*(\d+)\s+-----\s+([A-Za-z])-([A-Za-z]+)\s+potential\s+-----\s*$").unwrap();
    // ECP entry: "coeff r_exp g_exp"
    static ref ECP_ENTRY_RE: Regex = Regex::new(
        &format!(r"^\s*({})\s+(\d)\s+({})\s*$", helpers::FLOATING_RE.as_str(), helpers::FLOATING_RE.as_str())
    ).unwrap();
}

/// Parses lines representing all the electron shells for a single element.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    // First line is element name
    let parsed = helpers::parse_line_regex(&ELEMENT_BLOCK_RE, &basis_lines[0], "Element name")?;
    let element_name = parsed[0].to_lowercase();
    let element_Z = lut::element_Z_from_name(&element_name)
        .map_or(bse_raise!(ValueError, "Unknown element name: {}", element_name), Ok)?;

    let mut iline = 1;

    while iline < basis_lines.len() && SHELL_BLOCK_RE.is_match(&basis_lines[iline]) {
        let parsed = helpers::parse_line_regex(&SHELL_BLOCK_RE, &basis_lines[iline], "Shell AM, nprim")?;
        let am_char = &parsed[0];
        let nprim: usize = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid nprim: {}", parsed[1]), Ok)?;

        let shell_am = lut::amchar_to_int(am_char, true)
            .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?;

        let func_type = lut::function_type_from_am(&shell_am, "gto", "spherical");

        iline += 1;

        let mut exponents = Vec::new();
        let mut coefficients = Vec::new();

        for _ in 0..nprim {
            let parsed = helpers::parse_line_regex(&CONTRACTION_RE, &basis_lines[iline], "Contraction line")?;
            // let prim_idx: usize = parsed[0].parse().unwrap(); // Not used
            let expn = helpers::replace_d(&parsed[1]);
            let coeff = helpers::replace_d(&parsed[2]);

            // Skip zero coefficients
            if coeff.parse::<f64>().unwrap_or(0.0) != 0.0 {
                exponents.push(expn);
                coefficients.push(coeff);
            }

            iline += 1;
        }

        if !exponents.is_empty() {
            let shell = BseElectronShell {
                function_type: func_type,
                region: "".to_string(),
                angular_momentum: shell_am,
                exponents,
                coefficients: vec![coefficients],
            };

            elements
                .entry(element_Z.to_string())
                .or_default()
                .electron_shells
                .get_or_insert_with(Default::default)
                .push(shell);
        }
    }

    Ok(())
}

/// Parses lines representing all the ECP potentials for a single element.
fn parse_ecp_lines(elements: &mut HashMap<String, BseBasisElement>, basis_lines: &[String]) -> Result<(), BseError> {
    let mut iline = 0;

    while iline < basis_lines.len() && ECP_BLOCK_RE.is_match(&basis_lines[iline]) {
        let parsed = helpers::parse_line_regex(&ECP_BLOCK_RE, &basis_lines[iline], "ECP block")?;
        let element_sym = &parsed[0];
        let ecp_electrons: i32 = parsed[1].parse().unwrap();
        let _lmax: i32 = parsed[2].parse().unwrap();

        let element_Z = lut::element_Z_from_sym(element_sym)
            .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

        elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ecp_electrons);

        iline += 1;

        while iline < basis_lines.len() && ECP_SHELL_RE.is_match(&basis_lines[iline]) {
            let parsed = helpers::parse_line_regex(&ECP_SHELL_RE, &basis_lines[iline], "ECP shell")?;
            let nlines: usize = parsed[0].parse().unwrap();
            let am_char = &parsed[1];
            // let base_am = &parsed[2]; // Not used

            let pot_am = lut::amchar_to_int(am_char, true)
                .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?;

            iline += 1;

            let mut g_exp = Vec::new();
            let mut r_exp = Vec::new();
            let mut coeff = Vec::new();

            for _ in 0..nlines {
                let parsed = helpers::parse_line_regex(&ECP_ENTRY_RE, &basis_lines[iline], "ECP entry")?;
                let c = helpers::replace_d(&parsed[0]);
                let r: i32 = parsed[1].parse().unwrap();
                let g = helpers::replace_d(&parsed[2]);

                if c.parse::<f64>().unwrap_or(0.0) != 0.0 {
                    g_exp.push(g);
                    r_exp.push(r);
                    coeff.push(c);
                }

                iline += 1;
            }

            if !coeff.is_empty() {
                let ecp_pot = BseEcpPotential {
                    angular_momentum: pot_am,
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

    Ok(())
}

pub fn read_gamess_us(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    // Removes comments
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "!#$", true, true);

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

    // Split into element blocks (electron shells)
    let element_blocks = helpers::partition_lines(
        &basis_lines,
        |x| ELEMENT_BLOCK_RE.is_match(x) && lut::element_Z_from_name(&x.trim().to_lowercase()).is_some(),
        0,
        None,
        None,
        None,
        2,
        true,
    )?;

    // Split into ECP blocks
    let ecp_blocks =
        helpers::partition_lines(&basis_lines, |x| ECP_BLOCK_RE.is_match(x), 0, None, None, None, 2, true)?;

    for element_lines in element_blocks {
        // Skip if this looks like an ECP block
        if element_lines.iter().any(|x| ECP_BLOCK_RE.is_match(x)) {
            continue;
        }
        parse_electron_lines(&mut basis_dict.elements, &element_lines)?;
    }

    for ecp_lines in ecp_blocks {
        parse_ecp_lines(&mut basis_dict.elements, &ecp_lines)?;
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_gamess_us() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "gamess_us", args);
        let basis = read_gamess_us(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_gamess_us_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "gamess_us", args);
        let basis = read_gamess_us(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
