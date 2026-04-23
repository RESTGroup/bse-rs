//! Reader for the NWChem format

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    static ref AM_LINE_RE: Regex = Regex::new(r"^([A-Za-z]+)\s+([A-Za-z]+)$").unwrap();
    static ref NELEC_RE: Regex = RegexBuilder::new(r"^([a-z]+)\s+nelec\s+(\d+)$").case_insensitive(true).build().unwrap();
}

/// Parses lines representing all the electron shells for all elements.
fn parse_electron_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    basis_lines: &[String],
) -> Result<(), BseError> {
    // assumes that basis_lines are already trimmed

    // Remove 'end' from the lines (if they exist)
    // They may exist when this is called from other readers
    let basis_lines = basis_lines.iter().filter(|line| !line.eq_ignore_ascii_case("end")).cloned().collect_vec();

    // Basis entry needs to start with 'basis'
    if !basis_lines.first().is_some_and(|line| line.to_lowercase().starts_with("basis")) {
        return bse_raise!(ValueError, "Basis entry must start with 'basis'");
    }

    // Is the basis set spherical or cartesian?
    let am_type = if basis_lines[0].to_lowercase().contains("spherical") { "spherical" } else { "cartesian" };

    // Start at index 1 in order to strip of the first line ('BASIS AO PRINT' or
    // something)
    let shell_blocks = helpers::partition_lines(
        &basis_lines[1..],
        |x| x.chars().next().is_some_and(|c| c.is_alphabetic()),
        0,
        None,
        None,
        None,
        2,
        true,
    )?;

    for shl_lines in shell_blocks.iter() {
        let parsed = helpers::parse_line_regex(&AM_LINE_RE, shl_lines.first().unwrap(), "Element sym, shell am")?;
        let element_Z = lut::element_Z_from_sym(&parsed[0])
            .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", parsed[0]), Ok)?;
        let shell_am = lut::amchar_to_int(&parsed[1], HIK)
            .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", parsed[1]), Ok)?;

        let function_type = lut::function_type_from_am(&shell_am, "gto", am_type);

        // How many columns of coefficients do we have?
        // Only if this is a fused shell do we know.
        let ngen = if shell_am.len() > 1 { Some(shell_am.len()) } else { None };
        let (exponents, coefficients) = helpers::parse_primitive_matrix(&shl_lines[1..], None, ngen, None)?;

        let shell = BseElectronShell {
            function_type,
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

/// Parses lines representing all the ECP potentials for all elements.
fn parse_ecp_lines(elements: &mut HashMap<String, BseBasisElement>, basis_lines: &[String]) -> Result<(), BseError> {
    // Remove 'end' from the lines (if they exist)
    // They may exist when this is called from other readers
    let basis_lines = basis_lines.iter().filter(|line| !line.eq_ignore_ascii_case("end")).cloned().collect_vec();

    // Start at index 1 in order to strip of the first line ('ECP' or something).
    // This splits out based on starting with an alpha character. A block can be
    // either a potential or the nelec line.
    let ecp_blocks = helpers::partition_lines(
        &basis_lines[1..],
        |x| x.chars().next().is_some_and(|c| c.is_alphabetic()),
        0,
        None,
        None,
        None,
        1,
        true,
    )?;

    for pot_lines in ecp_blocks.iter().filter(|x| !x.is_empty()) {
        // Check if this is a nelec line
        if pot_lines.len() == 1 {
            let parsed = helpers::parse_line_regex(&NELEC_RE, &pot_lines[0], "ECP: Element sym, nelec")?;
            let element_Z = lut::element_Z_from_sym(&parsed[0])
                .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", parsed[0]), Ok)?;
            let nelec = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid nelec value: {}", parsed[1]), Ok)?;

            elements.entry(element_Z.to_string()).or_default().ecp_electrons = Some(nelec);
        } else {
            let parsed = helpers::parse_line_regex(&AM_LINE_RE, &pot_lines[0], "ECP: Element sym, pot AM")?;
            let element_Z = lut::element_Z_from_sym(&parsed[0])
                .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", parsed[0]), Ok)?;
            let pot_am = match parsed[1].to_lowercase().as_str() {
                "ul" => vec![],
                _ => lut::amchar_to_int(&parsed[1], HIK)
                    .map_or(bse_raise!(ValueError, "Unknown angular momentum: {}", parsed[1]), Ok)?,
            };
            let ecp_data = helpers::parse_ecp_table(&pot_lines[1..], &["r_exp", "g_exp", "coeff"], None)?;
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
        };
    }

    for (k, v) in elements.iter_mut().filter(|(_, v)| v.ecp_potentials.is_some()) {
        // Fix ecp angular momentum now that everything has been read
        // Specifically, we can set the 'ul' potential to be the max am + 1
        let ecp_potentials = v.ecp_potentials.as_mut().unwrap();
        let max_ecp_am = ecp_potentials.iter().flat_map(|p| &p.angular_momentum).max().cloned().unwrap_or(0);
        ecp_potentials.iter_mut().for_each(|p| {
            if p.angular_momentum.is_empty() {
                p.angular_momentum = vec![max_ecp_am + 1];
            }
        });

        // Make sure the number of electrons replaced by the ECP was specified for all
        // elements
        if v.ecp_electrons.is_none() {
            bse_raise!(ValueError, "Number of ECP electrons not specified for element {k}")?;
        }
    }

    Ok(())
}

pub fn read_nwchem(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    let lines: Vec<String> =
        basis_str.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && !s.starts_with('#')).collect();

    let mut basis_dict = BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "minimal".to_string(), schema_version: "0.1".to_string() },
        elements: HashMap::new(),
        function_types: Vec::new(),
        name: "unknown_basis".to_string(),
        description: "no_description".to_string(),
    };

    let basis_sections =
        helpers::partition_lines(&lines, |x| x.to_lowercase() == "end", 0, None, Some(1), Some(2), 1, false)?;

    for s in basis_sections.iter().filter(|s| !s.is_empty()) {
        if s[0].to_lowercase().starts_with("basis") {
            parse_electron_lines(&mut basis_dict.elements, s)?
        } else if s[0].to_lowercase().starts_with("ecp") {
            parse_ecp_lines(&mut basis_dict.elements, s)?
        } else {
            return bse_raise!(ValueError, "Unknown section in NWChem basis: {}", s[0]);
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
    fn test_read_nwchem() {
        let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "nwchem", args);
        let basis = read_nwchem(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
