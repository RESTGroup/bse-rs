//! Reader for the Molpro format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Basis entry start: 'basis={' allowing whitespace
    static ref BASIS_START_RE: Regex = Regex::new(r"^\s*basis\s*=\s*\{\s*$").unwrap();
    // Basis ends with '}'
    static ref BASIS_END_RE: Regex = Regex::new(r"^\s*\}\s*$").unwrap();
    // Shell entry: 'am,element,expn1,expn2,...' allowing whitespace
    static ref ELEMENT_SHELL_RE: Regex = Regex::new(
        &format!(r"^\s*([spdfghikSPDFGHIK])\s*,?\s*(\w+)\s*(?:,?\s*({})\s*)+\s*$",
        helpers::FLOATING_RE.as_str())
    ).unwrap();
    // Contraction entry: 'c,start.end,coeff1,coeff2,...'
    static ref CONTRACTION_RE: Regex = Regex::new(
        &format!(r"^\s*c\s*,?\s*(\d+)\.(\d+)\s*(?:,?\s*({})\s*)+\s*$",
        helpers::FLOATING_RE.as_str())
    ).unwrap();
    // ECP entry: ECP, symbol, number of electrons in ECP, lmax
    static ref ECP_RE: Regex = Regex::new(
        r"^\s*ECP\s*,\s*(\w+)\s*,\s*(\d+)\s*,\s*(\d+)\s*;\s*$"
    ).unwrap();
    // ECP block start: number of terms
    static ref ECP_BLOCK_RE: Regex = Regex::new(r"^\s*(\d+)\s*;").unwrap();
    // ECP data: rexp expn coeff
    static ref ECP_DATA_RE: Regex = Regex::new(
        &format!(r"^\s*(\d+)\s*,\s*({})\s*,\s*({})\s*;\s*", helpers::FLOATING_RE.as_str(), helpers::FLOATING_RE.as_str())
    ).unwrap();
}

/// Reads a shell from the input lines.
fn read_shell(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
    iline: usize,
    func_type: &str,
) -> Result<usize, BseError> {
    // Read the shell entry
    let line = &basis_lines[iline];
    let caps = ELEMENT_SHELL_RE
        .captures(line)
        .map_or(bse_raise!(ValueError, "Shell entry does not match regex: {}", line), Ok)?;

    // Angular momentum
    let am_char = caps.get(1).unwrap().as_str();
    let shell_am = lut::amchar_to_int(am_char, true)
        .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", am_char), Ok)?;

    // Element
    let element_sym = caps.get(2).unwrap().as_str();
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Parse exponents from the line manually (since regex repeated groups only
    // capture last) Format: am, element, exp1, exp2, ...
    let parts: Vec<&str> = line.split(',').collect();
    let exponents: Vec<String> = parts
        .iter()
        .skip(2) // Skip am and element
        .map(|s| helpers::replace_d(s.trim()))
        .collect();

    let nprim = exponents.len();
    if nprim == 0 {
        bse_raise!(ValueError, "No exponents found for shell")?;
    }

    // Read in contractions
    let mut coefficients: Vec<Vec<String>> = Vec::new();
    let mut current_line = iline + 1;

    while current_line < basis_lines.len() && CONTRACTION_RE.is_match(&basis_lines[current_line]) {
        let line = &basis_lines[current_line];

        // Parse manually: c, start.end, coeff1, coeff2, ...
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            bse_raise!(ValueError, "Invalid contraction line: {}", line)?;
        }

        // Parse start.end
        let range_str = parts[1].trim();
        let range_parts: Vec<&str> = range_str.split('.').collect();
        if range_parts.len() != 2 {
            bse_raise!(ValueError, "Invalid range in contraction: {}", range_str)?;
        }
        let start: usize =
            range_parts[0].parse().map_or(bse_raise!(ValueError, "Invalid start: {}", range_parts[0]), Ok)?;
        let end: usize =
            range_parts[1].parse().map_or(bse_raise!(ValueError, "Invalid end: {}", range_parts[1]), Ok)?;

        // Parse coefficients
        let cc: Vec<String> = parts.iter().skip(2).map(|s| helpers::replace_d(s.trim())).collect();

        let ncontr = end - start + 1;
        if cc.len() != ncontr {
            bse_raise!(
                ValueError,
                "Number of coefficients ({}) does not match range ({} to {})",
                cc.len(),
                start,
                end
            )?;
        }

        // Pad coefficients with zeros
        let mut padded = Vec::new();
        if start > 1 {
            for _ in 1..start {
                padded.push("0.0".to_string());
            }
        }
        padded.extend(cc.clone());
        if end < nprim {
            for _ in end..nprim {
                padded.push("0.0".to_string());
            }
        }

        if padded.len() != nprim {
            bse_raise!(ValueError, "Padded coefficients length {} != nprim {}", padded.len(), nprim)?;
        }

        coefficients.push(padded);
        current_line += 1;
    }

    // Determine function type
    let func_type = if shell_am[0] < 2 { "gto".to_string() } else { func_type.to_string() };

    // Store the data
    let shell = BseElectronShell {
        function_type: func_type,
        region: "".to_string(),
        angular_momentum: shell_am,
        exponents,
        coefficients,
    };

    elements.entry(element_Z.to_string()).or_default().electron_shells.get_or_insert_with(Default::default).push(shell);

    Ok(current_line)
}

/// Reads an ECP from the input lines.
fn read_ecp(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
    iline: usize,
) -> Result<usize, BseError> {
    // Read the ECP entry
    let caps = ECP_RE
        .captures(&basis_lines[iline])
        .map_or(bse_raise!(ValueError, "ECP entry does not match regex: {}", basis_lines[iline]), Ok)?;

    let element_sym = caps.get(1).unwrap().as_str();
    let ncore: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
    let lmax: i32 = caps.get(3).unwrap().as_str().parse().unwrap();

    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Set ECP electrons
    elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(ncore);

    let mut current_line = iline + 1;

    // Read ECP blocks
    for il in -1..lmax {
        let ecp_l = if il == -1 { lmax } else { il };

        // Read the number of terms in the block
        let caps = ECP_BLOCK_RE
            .captures(&basis_lines[current_line])
            .map_or(bse_raise!(ValueError, "ECP block does not match regex: {}", basis_lines[current_line]), Ok)?;
        let nterms: usize = caps.get(1).unwrap().as_str().parse().unwrap();
        current_line += 1;

        let mut r_exp = Vec::new();
        let mut g_exp = Vec::new();
        let mut coeff = Vec::new();

        for _ in 0..nterms {
            let caps = ECP_DATA_RE
                .captures(&basis_lines[current_line])
                .map_or(bse_raise!(ValueError, "ECP data does not match regex: {}", basis_lines[current_line]), Ok)?;
            r_exp.push(caps.get(1).unwrap().as_str().parse::<i32>().unwrap() + 2); // Molpro uses bare exponent, BSE uses r^{-2} prefactor
            g_exp.push(helpers::replace_d(caps.get(2).unwrap().as_str()));
            coeff.push(helpers::replace_d(caps.get(3).unwrap().as_str()));
            current_line += 1;
        }

        let ecp_pot = BseEcpPotential {
            angular_momentum: vec![ecp_l],
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

    Ok(current_line)
}

/// Parses lines representing all the electron shells for all elements.
fn parse_lines(
    basis_lines: &[String],
    elements: &mut HashMap<String, BseBasisElement>,
    func_type: &str,
) -> Result<(), BseError> {
    let mut iline = 0;

    while iline < basis_lines.len() {
        if ELEMENT_SHELL_RE.is_match(&basis_lines[iline]) {
            iline = read_shell(elements, basis_lines, iline, func_type)?;
        } else if ECP_RE.is_match(&basis_lines[iline]) {
            iline = read_ecp(elements, basis_lines, iline)?;
        } else {
            iline += 1;
        }
    }

    Ok(())
}

pub fn read_molpro(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    // Removes comments
    let basis_lines =
        helpers::prune_lines(&basis_str.lines().map(|s| s.trim().to_string()).collect_vec(), "!*", true, true);

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

    // Determine function type (spherical by default)
    let mut func_type = "gto_spherical";
    for line in &basis_lines {
        let trimmed = line.trim().to_lowercase();
        if trimmed == "spherical" {
            func_type = "gto_spherical";
        } else if trimmed == "cartesian" {
            func_type = "gto_cartesian";
        }
    }

    parse_lines(&basis_lines, &mut basis_dict.elements, func_type)?;

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_molpro() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "molpro", args);
        let basis = read_molpro(&basis_str).unwrap();
        println!("{basis:#?}");
    }

    #[test]
    fn test_read_molpro_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("49-51".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("def2-ECP", "molpro", args);
        let basis = read_molpro(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
