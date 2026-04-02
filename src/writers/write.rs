//! Driver for converting basis set data to a specified output format.

use crate::prelude::*;

/// Check if format is a directory format (starts with "dir-").
pub fn is_dir_format(fmt: &str) -> bool {
    fmt.to_lowercase().starts_with("dir-")
}

/// Strip "dir-" prefix from format name to get underlying format.
///
/// Returns the format name without the "dir-" prefix.
/// If the format doesn't start with "dir-", returns it unchanged.
pub fn strip_dir_prefix(fmt: &str) -> &str {
    let fmt_lower = fmt.to_lowercase();
    if fmt_lower.starts_with("dir-") {
        &fmt[4..]
    } else {
        fmt
    }
}

/// Format information for a writer.
#[allow(dead_code)]
struct WriterFormat {
    display: &'static str,
    extension: &'static str,
    comment: &'static str,
    valid: Vec<&'static str>,
}

fn writer_format_map() -> HashMap<&'static str, WriterFormat> {
    HashMap::from([
        ("nwchem", WriterFormat {
            display: "NWChem",
            extension: ".nw",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("gaussian94", WriterFormat {
            display: "Gaussian",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("gaussian94lib", WriterFormat {
            display: "Gaussian, system library",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("psi4", WriterFormat {
            display: "Psi4",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("molcas", WriterFormat {
            display: "Molcas",
            extension: ".molcas",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("qchem", WriterFormat {
            display: "Q-Chem",
            extension: ".qchem",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("orca", WriterFormat {
            display: "ORCA",
            extension: ".orca",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("dalton", WriterFormat {
            display: "Dalton",
            extension: ".dalton",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("qcschema", WriterFormat {
            display: "QCSchema",
            extension: ".json",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("cp2k", WriterFormat {
            display: "CP2K",
            extension: ".cp2k",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("pqs", WriterFormat {
            display: "PQS",
            extension: ".pqs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("demon2k", WriterFormat {
            display: "deMon2K",
            extension: ".d2k",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("gamess_us", WriterFormat {
            display: "GAMESS US",
            extension: ".bas",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("turbomole", WriterFormat {
            display: "Turbomole",
            extension: ".tm",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("gamess_uk", WriterFormat {
            display: "GAMESS UK",
            extension: ".bas",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("molpro", WriterFormat {
            display: "Molpro",
            extension: ".mpro",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("cfour", WriterFormat {
            display: "CFOUR",
            extension: ".c4bas",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("acesii", WriterFormat {
            display: "ACES II",
            extension: ".acesii",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("xtron", WriterFormat {
            display: "xTron",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("bdf", WriterFormat {
            display: "BDF",
            extension: ".bdf",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("fhiaims", WriterFormat {
            display: "FHI-aims",
            extension: ".fhiaims",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical"],
        }),
        ("jaguar", WriterFormat {
            display: "Jaguar",
            extension: ".jaguar",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("crystal", WriterFormat {
            display: "CRYSTAL",
            extension: ".crystal",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("veloxchem", WriterFormat {
            display: "VeloxChem",
            extension: ".vlx",
            comment: "!",
            valid: vec!["gto", "gto_spherical"],
        }),
        ("molcas_library", WriterFormat {
            display: "Molcas basis library",
            extension: ".molcas",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("libmol", WriterFormat {
            display: "Molpro system library",
            extension: ".libmol",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("bsedebug", WriterFormat {
            display: "BSE Debug",
            extension: ".debug",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("bsejson", WriterFormat {
            display: "BSE JSON",
            extension: ".json",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("json", WriterFormat {
            display: "JSON",
            extension: ".json",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        }),
        ("ricdwrap", WriterFormat {
            display: "acCD auxiliary basis wrapper",
            extension: ".molcas",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical"],
        }),
    ])
}

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
        "orca" => Some(Writer {
            display: "ORCA",
            extension: ".orca",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::orca::write_orca,
        }),
        "dalton" => Some(Writer {
            display: "Dalton",
            extension: ".dalton",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::dalton::write_dalton,
        }),
        "qcschema" => Some(Writer {
            display: "QCSchema",
            extension: ".json",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::qcschema::write_qcschema,
        }),
        "cp2k" => Some(Writer {
            display: "CP2K",
            extension: ".cp2k",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::cp2k::write_cp2k,
        }),
        "pqs" => Some(Writer {
            display: "PQS",
            extension: ".pqs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::pqs::write_pqs,
        }),
        "demon2k" => Some(Writer {
            display: "deMon2K",
            extension: ".d2k",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::demon2k::write_demon2k,
        }),
        "gamess_us" => Some(Writer {
            display: "GAMESS US",
            extension: ".bas",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::gamess_us::write_gamess_us,
        }),
        "turbomole" => Some(Writer {
            display: "Turbomole",
            extension: ".tm",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::turbomole::write_turbomole,
        }),
        "gamess_uk" => Some(Writer {
            display: "GAMESS UK",
            extension: ".bas",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::gamess_uk::write_gamess_uk,
        }),
        "molpro" => Some(Writer {
            display: "Molpro",
            extension: ".mpro",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::molpro::write_molpro,
        }),
        "cfour" => Some(Writer {
            display: "CFOUR",
            extension: ".c4bas",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::genbas::write_cfour,
        }),
        "acesii" => Some(Writer {
            display: "ACES II",
            extension: ".acesii",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::genbas::write_aces2,
        }),
        "xtron" => Some(Writer {
            display: "xTron",
            extension: ".gbs",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::g94::write_xtron,
        }),
        "bdf" => Some(Writer {
            display: "BDF",
            extension: ".bdf",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::bdf::write_bdf,
        }),
        "fhiaims" => Some(Writer {
            display: "FHI-aims",
            extension: ".fhiaims",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical"],
            function: writers::fhiaims::write_fhiaims,
        }),
        "jaguar" => Some(Writer {
            display: "Jaguar",
            extension: ".jaguar",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::jaguar::write_jaguar,
        }),
        "crystal" => Some(Writer {
            display: "CRYSTAL",
            extension: ".crystal",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::crystal::write_crystal,
        }),
        "veloxchem" => Some(Writer {
            display: "VeloxChem",
            extension: ".vlx",
            comment: "!",
            valid: vec!["gto", "gto_spherical"],
            function: writers::veloxchem::write_veloxchem,
        }),
        "molcas_library" => Some(Writer {
            display: "Molcas basis library",
            extension: ".molcas",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::molcas_library::write_molcas_library,
        }),
        "libmol" => Some(Writer {
            display: "Molpro system library",
            extension: ".libmol",
            comment: "!",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::libmol::write_libmol,
        }),
        "bsedebug" => Some(Writer {
            display: "BSE Debug",
            extension: ".debug",
            comment: "#",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::bsedebug::write_bsedebug,
        }),
        "bsejson" | "json" => Some(Writer {
            display: "BSE JSON",
            extension: ".json",
            comment: "",
            valid: vec!["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
            function: writers::bsejson::write_bsejson,
        }),
        "ricdwrap" => Some(Writer {
            display: "acCD auxiliary basis wrapper",
            extension: ".molcas",
            comment: "*",
            valid: vec!["gto", "gto_cartesian", "gto_spherical"],
            function: writers::ricdwrap::write_ricdwrap,
        }),
        _ => None,
    }
}

/// Return information about the basis set formats available for writing.
///
/// The returned data is a map of format name to display name. The format
/// name can be passed as the `fmt` argument to [`write_formatted_basis_str`].
///
/// # Arguments
///
/// * `function_types` - Optional list of function types to filter by. If
///   provided, only formats supporting those types are returned. Example:
///   `["gto", "gto_spherical"]`
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let formats = get_writer_formats(None);
/// assert!(!formats.is_empty());
/// assert!(formats.contains_key("nwchem"));
/// println!("Available writer formats: {:?}", formats);
/// ```
pub fn get_writer_formats(function_types: Option<Vec<String>>) -> HashMap<String, String> {
    let format_map = writer_format_map();

    if function_types.is_none() {
        return format_map.into_iter().map(|(k, v)| (k.to_string(), v.display.to_string())).collect();
    }

    let ftypes: HashSet<String> = function_types.unwrap().into_iter().map(|s| s.to_lowercase()).collect();
    let mut ret = HashMap::new();

    for (fmt, v) in format_map {
        let valid_types: HashSet<String> = v.valid.iter().map(|s| s.to_string()).collect();
        if ftypes.is_subset(&valid_types) {
            ret.insert(fmt.to_string(), v.display.to_string());
        }
    }

    ret
}

/// Returns the recommended file extension for a given format.
///
/// # Arguments
///
/// * `fmt` - The format name (case insensitive)
///
/// # Returns
///
/// The recommended file extension (e.g., ".nw" for nwchem).
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let ext = get_format_extension("nwchem").unwrap();
/// assert_eq!(ext, ".nw");
/// ```
pub fn get_format_extension(fmt: &str) -> Result<&'static str, BseError> {
    let fmt = fmt.to_lowercase();
    let format_map = writer_format_map();
    if !format_map.contains_key(fmt.as_str()) {
        bse_raise!(ValueError, "Unknown basis set format '{}'", fmt)?;
    }
    Ok(format_map[fmt.as_str()].extension)
}

/// Returns the basis set data as a string representing the data in the
/// specified output format.
pub fn write_formatted_basis_str(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> String {
    write_formatted_basis_str_f(basis_dict, fmt, header).unwrap()
}

pub fn write_formatted_basis_str_f(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> Result<String, BseError> {
    // make writers case insensitive
    let fmt = fmt.to_lowercase();
    let writer = writer_map(&fmt).map_or(bse_raise!(ValueError, "Unknown writer format: {fmt}"), Ok)?;

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

    if let Some(header) = header {
        if !writer.comment.is_empty() {
            let comment_str = writer.comment;
            let header_str = header.split('\n').map(|line| format!("{comment_str}{line}")).join("\n");

            // HACK - Gaussian94Lib doesn't tolerate blank lines after the header
            if fmt == "gaussian94lib" {
                ret_str.insert_str(0, &header_str);
            } else {
                ret_str.insert_str(0, &format!("{header_str}\n\n"));
            }
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
