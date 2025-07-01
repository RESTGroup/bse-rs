//! Driver for converting basis set data to a specified output format.

use crate::prelude::*;

#[allow(dead_code)]
struct Writer {
    display: &'static str,
    extension: &'static str,
    comment: &'static str,
    valid: Vec<&'static str>,
    function: fn(&BseBasis) -> String,
}

fn writer_map(fmt: &str) -> Option<Writer> {
    let fmt = fmt.to_lowercase();
    match fmt.as_str() {
        "nwchem" => Some(Writer {
            display: "NWChem",
            extension: ".nw",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::nwchem::write_nwchem,
        }),
        "gaussian94" | "g94" => Some(Writer {
            display: "Gaussian",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::g94::write_g94,
        }),
        "gaussian94lib" | "g94lib" => Some(Writer {
            display: "Gaussian, system library",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::g94::write_g94lib,
        }),
        "psi4" => Some(Writer {
            display: "Psi4",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::g94::write_psi4,
        }),
        "xtron" => Some(Writer {
            display: "xTron",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::g94::write_xtron,
        }),
        "molcas" => Some(Writer {
            display: "Molcas",
            extension: ".molcas",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::molcas::write_molcas,
        }),
        "qchem" => Some(Writer {
            display: "Q-Chem",
            extension: ".qchem",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::qchem::write_qchem,
        }),
        _ => None,
    }
}

/// Returns the basis set data as a string representing the data in the
/// specified output format.
pub fn write_formatted_basis_str(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> String {
    write_formatted_basis_str_f(basis_dict, fmt, header).unwrap()
}

pub fn write_formatted_basis_str_f(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> Result<String, BseError> {
    // make writers case insensitive
    let fmt = fmt.to_lowercase();
    let writer = writer_map(&fmt).map_or(bse_raise!(ValueError, "Unknown format: {fmt}"), Ok)?;

    // Determine if the writer supports all the types in the basis_dict
    if !writer.valid.is_empty() {
        let ftypes: HashSet<String> = basis_dict.function_types.iter().cloned().collect();
        let valid_types: HashSet<String> = writer.valid.iter().map(|s| s.to_string()).collect();
        if !ftypes.is_subset(&valid_types) {
            bse_raise!(
                ValueError,
                "Writer {} does not support all function types: {:?} vs {:?}",
                writer.display,
                basis_dict.function_types,
                writer.valid
            )?
        }
    }

    // Actually do the conversion
    let mut ret_str = (writer.function)(basis_dict);

    if let Some(header) = header
        && !writer.comment.is_empty()
    {
        let comment_str = writer.comment;
        let header_str = header.split('\n').map(|line| format!("{comment_str}{line}")).join("\n");

        // HACK - Gaussian94Lib doesn't tolerate blank lines after the header
        if fmt == "gaussian94lib" {
            ret_str.insert_str(0, &header_str);
        } else {
            ret_str.insert_str(0, &format!("{header_str}\n\n"));
        }
    }

    // HACK - Psi4 requires the first non-comment line be spherical/cartesian, so we
    // have to add that before the header
    if fmt == "psi4" {
        let types = &basis_dict.function_types;
        let harm_type = if types.contains(&"gto_cartesian".to_string()) { "cartesian" } else { "spherical" };
        ret_str.insert_str(0, &format!("{harm_type}\n\n"));
    }

    Ok(ret_str)
}
