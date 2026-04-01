//! Reader for VeloxChem format.

use crate::prelude::*;
use crate::readers::helpers;

lazy_static::lazy_static! {
    // Shell line: "S    5    1" (amchar, nprim, ncont)
    // Note: VeloxChem uses 'hij' convention where AM=7 is J
    static ref SHELL_BEGIN_RE: Regex = Regex::new(r"^([SPDFGHIJKLMNOQRTUVWXYZABCE])\s+(\d+)\s+(\d+)$").unwrap();
}

/// Parses lines representing all the shells for a single element.
fn parse_element_lines(
    elements: &mut HashMap<String, BseBasisElement>,
    element_lines: &[String],
    element_sym: &str,
) -> Result<(), BseError> {
    let element_Z = lut::element_Z_from_sym(element_sym)
        .map_or(bse_raise!(ValueError, "Unknown element symbol: {}", element_sym), Ok)?;

    // Partition into shell blocks
    let shell_blocks = helpers::partition_lines(
        element_lines,
        |x| SHELL_BEGIN_RE.is_match(x),
        0,
        None,
        None,
        None,
        2, // minimum size: shell header + at least one data line
        true,
    )?;

    for shell_lines in shell_blocks {
        // Parse shell header: amchar, nprim, ncont
        let parsed = helpers::parse_line_regex(&SHELL_BEGIN_RE, &shell_lines[0], "Shell: amchar, nprim, ncont")?;
        let amchar = &parsed[0];
        let nprim: usize = parsed[1].parse().map_or(bse_raise!(ValueError, "Invalid nprim: {}", parsed[1]), Ok)?;
        let ncont: usize = parsed[2].parse().map_or(bse_raise!(ValueError, "Invalid ncont: {}", parsed[2]), Ok)?;

        // VeloxChem format requires ncont == 1 (uncontracted)
        if ncont != 1 {
            eprintln!("Warning: VeloxChem format expects ncont=1 for all shells, found ncont={ncont}");
        }

        // Parse exponents and coefficients from remaining lines
        let (exponents, coefficients) =
            helpers::parse_primitive_matrix(&shell_lines[1..], Some(nprim), Some(ncont), None)?;

        // Convert angular momentum character to integer (using hij convention)
        let am = lut::amchar_to_int(amchar, HIJ).unwrap_or_default();
        let func_type = lut::function_type_from_am(&am, "gto", "spherical");

        let shell = BseElectronShell {
            function_type: func_type,
            region: "".to_string(),
            angular_momentum: am,
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

pub fn read_veloxchem(basis_str: &str) -> Result<BseBasisMinimal, BseError> {
    let mut basis_lines: Vec<String> = basis_str.lines().map(|s| s.trim().to_string()).collect();

    // Empty file?
    if basis_lines.is_empty() {
        return Ok(BseBasisMinimal {
            molssi_bse_schema: BseMolssiBseSchema {
                schema_type: "minimal".to_string(),
                schema_version: "0.1".to_string(),
            },
            elements: HashMap::new(),
            function_types: Vec::new(),
            name: "unknown_basis".to_string(),
            description: "no_description".to_string(),
        });
    }

    // Read expected MD5 checksum (last line)
    let expected_md5sum = basis_lines.pop().unwrap_or_default().trim().to_string();

    // Find the @BASIS_SET line
    let basis_set_idx = basis_lines.iter().position(|x| x.starts_with("@BASIS_SET"));
    let basis_set_idx = match basis_set_idx {
        Some(idx) => idx,
        None => bse_raise!(ValueError, "No @BASIS_SET line found in VeloxChem format")?,
    };

    // Validate MD5 checksum (optional warning)
    let content_to_check = basis_lines[basis_set_idx..].join("\n") + "\n";
    let computed_md5sum = format!("{:x}", md5::compute(content_to_check.as_bytes()));
    if computed_md5sum != expected_md5sum {
        eprintln!(
            "Warning: VeloxChem MD5 checksum mismatch (computed: {}, expected: {})",
            computed_md5sum, expected_md5sum
        );
    }

    // Prune comments and blank lines
    basis_lines = helpers::prune_lines(&basis_lines, "!#", true, true);

    // Get @ATOMBASIS and @END markers
    let atombasis_indices: Vec<(usize, String)> = basis_lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if line.starts_with("@ATOMBASIS") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((i, parts[1].to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let end_indices: Vec<usize> = basis_lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| if line.starts_with("@END") { Some(i) } else { None })
        .collect();

    if atombasis_indices.len() != end_indices.len() {
        bse_raise!(ValueError, "Mismatched @ATOMBASIS and @END markers")?;
    }

    let mut basis_dict = BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "minimal".to_string(), schema_version: "0.1".to_string() },
        elements: HashMap::new(),
        function_types: Vec::new(),
        name: "unknown_basis".to_string(),
        description: "no_description".to_string(),
    };

    // Parse each element block
    for ((start_idx, element_sym), end_idx) in atombasis_indices.into_iter().zip(end_indices.into_iter()) {
        if end_idx <= start_idx + 1 {
            continue;
        }
        let element_lines: Vec<String> = basis_lines[start_idx + 1..end_idx].to_vec();
        parse_element_lines(&mut basis_dict.elements, &element_lines, &element_sym)?;
    }

    let function_types = compose::whole_basis_types(&basis_dict.elements);
    basis_dict.function_types = function_types;

    Ok(basis_dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_veloxchem() {
        let args = BseGetBasisArgsBuilder::default().elements("H, C-O".to_string()).build().unwrap();
        let basis_str = get_formatted_basis("cc-pVDZ", "veloxchem", args);
        let basis = read_veloxchem(&basis_str).unwrap();
        println!("{basis:#?}");
    }
}
