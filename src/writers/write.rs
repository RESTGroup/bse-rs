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

/// Writer format information.
#[derive(Clone, Copy)]
pub struct WriterInfo {
    /// Canonical format name
    pub name: &'static str,
    /// Display name for the format
    pub display: &'static str,
    /// File extension for the format
    pub extension: &'static str,
    /// Comment character for the format
    pub comment: &'static str,
    /// Valid function types for this format
    pub valid: &'static [&'static str],
    /// Aliases for this format (including extension if unique)
    pub aliases: &'static [&'static str],
}

impl WriterInfo {
    /// Get the file extension without the leading dot.
    pub fn extension_without_dot(&self) -> &'static str {
        self.extension.trim_start_matches('.')
    }

    /// Check if the given function types are valid for this format.
    pub fn supports_function_types(&self, ftypes: &HashSet<String>) -> bool {
        if self.valid.is_empty() {
            return true;
        }
        let valid_types: HashSet<String> = self.valid.iter().map(|s| s.to_string()).collect();
        ftypes.is_subset(&valid_types)
    }

    /// Check if a given name is an alias for this format.
    pub fn is_alias(&self, name: &str) -> bool {
        self.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
    }
}

/// Writer entry with all format information.
#[derive(Clone, Copy)]
struct WriterEntry {
    /// Canonical format name
    name: &'static str,
    /// Display name for the format
    display: &'static str,
    /// File extension for the format
    extension: &'static str,
    /// Comment character for the format
    comment: &'static str,
    /// Valid function types for this format
    valid: &'static [&'static str],
    /// All valid names for this format (canonical + aliases)
    names: &'static [&'static str],
    /// Writer function
    function: fn(&BseBasis) -> String,
}

/// All writer format entries.
const WRITER_ENTRIES: &[WriterEntry] = &[
    // nwchem - unique extension .nw
    WriterEntry {
        name: "nwchem",
        display: "NWChem",
        extension: ".nw",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["nwchem", "nw"],
        function: writers::nwchem::write_nwchem,
    },
    // gaussian94 - aliases: g94; extension .gbs (shared with gaussian94lib, psi4, xtron)
    WriterEntry {
        name: "gaussian94",
        display: "Gaussian",
        extension: ".gbs",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["gaussian94", "g94"],
        function: writers::g94::write_g94,
    },
    // gaussian94lib - aliases: g94lib
    WriterEntry {
        name: "gaussian94lib",
        display: "Gaussian, system library",
        extension: ".gbs",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["gaussian94lib", "g94lib"],
        function: writers::g94::write_g94lib,
    },
    // psi4 - extension .gbs (shared)
    WriterEntry {
        name: "psi4",
        display: "Psi4",
        extension: ".gbs",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["psi4"],
        function: writers::g94::write_psi4,
    },
    // molcas - unique extension .molcas
    WriterEntry {
        name: "molcas",
        display: "Molcas",
        extension: ".molcas",
        comment: "*",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["molcas"],
        function: writers::molcas::write_molcas,
    },
    // qchem - unique extension .qchem
    WriterEntry {
        name: "qchem",
        display: "Q-Chem",
        extension: ".qchem",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["qchem"],
        function: writers::qchem::write_qchem,
    },
    // orca - unique extension .orca
    WriterEntry {
        name: "orca",
        display: "ORCA",
        extension: ".orca",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["orca"],
        function: writers::orca::write_orca,
    },
    // dalton - unique extension .dalton
    WriterEntry {
        name: "dalton",
        display: "Dalton",
        extension: ".dalton",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["dalton"],
        function: writers::dalton::write_dalton,
    },
    // qcschema - extension .json (shared with bsejson)
    WriterEntry {
        name: "qcschema",
        display: "QCSchema",
        extension: ".json",
        comment: "",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["qcschema"],
        function: writers::qcschema::write_qcschema,
    },
    // cp2k - unique extension .cp2k
    WriterEntry {
        name: "cp2k",
        display: "CP2K",
        extension: ".cp2k",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["cp2k"],
        function: writers::cp2k::write_cp2k,
    },
    // pqs - unique extension .pqs
    WriterEntry {
        name: "pqs",
        display: "PQS",
        extension: ".pqs",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["pqs"],
        function: writers::pqs::write_pqs,
    },
    // demon2k - unique extension .d2k
    WriterEntry {
        name: "demon2k",
        display: "deMon2K",
        extension: ".d2k",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["demon2k", "d2k"],
        function: writers::demon2k::write_demon2k,
    },
    // gamess_us - unique extension .bas (shared with gamess_uk)
    WriterEntry {
        name: "gamess_us",
        display: "GAMESS US",
        extension: ".bas",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["gamess_us"],
        function: writers::gamess_us::write_gamess_us,
    },
    // turbomole - unique extension .tm
    WriterEntry {
        name: "turbomole",
        display: "Turbomole",
        extension: ".tm",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["turbomole", "tm"],
        function: writers::turbomole::write_turbomole,
    },
    // gamess_uk - extension .bas (shared with gamess_us)
    WriterEntry {
        name: "gamess_uk",
        display: "GAMESS UK",
        extension: ".bas",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["gamess_uk"],
        function: writers::gamess_uk::write_gamess_uk,
    },
    // molpro - unique extension .mpro
    WriterEntry {
        name: "molpro",
        display: "Molpro",
        extension: ".mpro",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["molpro", "mpro"],
        function: writers::molpro::write_molpro,
    },
    // cfour - unique extension .c4bas
    WriterEntry {
        name: "cfour",
        display: "CFOUR",
        extension: ".c4bas",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["cfour", "c4bas"],
        function: writers::genbas::write_cfour,
    },
    // acesii - unique extension .acesii
    WriterEntry {
        name: "acesii",
        display: "ACES II",
        extension: ".acesii",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["acesii"],
        function: writers::genbas::write_aces2,
    },
    // xtron - extension .gbs (shared)
    WriterEntry {
        name: "xtron",
        display: "xTron",
        extension: ".gbs",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["xtron"],
        function: writers::g94::write_xtron,
    },
    // bdf - unique extension .bdf
    WriterEntry {
        name: "bdf",
        display: "BDF",
        extension: ".bdf",
        comment: "*",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["bdf"],
        function: writers::bdf::write_bdf,
    },
    // fhiaims - unique extension .fhiaims
    WriterEntry {
        name: "fhiaims",
        display: "FHI-aims",
        extension: ".fhiaims",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical"],
        names: &["fhiaims"],
        function: writers::fhiaims::write_fhiaims,
    },
    // jaguar - unique extension .jaguar
    WriterEntry {
        name: "jaguar",
        display: "Jaguar",
        extension: ".jaguar",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["jaguar"],
        function: writers::jaguar::write_jaguar,
    },
    // crystal - unique extension .crystal
    WriterEntry {
        name: "crystal",
        display: "CRYSTAL",
        extension: ".crystal",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["crystal"],
        function: writers::crystal::write_crystal,
    },
    // veloxchem - unique extension .vlx
    WriterEntry {
        name: "veloxchem",
        display: "VeloxChem",
        extension: ".vlx",
        comment: "!",
        valid: &["gto", "gto_spherical"],
        names: &["veloxchem", "vlx"],
        function: writers::veloxchem::write_veloxchem,
    },
    // molcas_library - extension .molcas (shared with molcas)
    WriterEntry {
        name: "molcas_library",
        display: "Molcas basis library",
        extension: ".molcas",
        comment: "",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["molcas_library"],
        function: writers::molcas_library::write_molcas_library,
    },
    // libmol - unique extension .libmol
    WriterEntry {
        name: "libmol",
        display: "Molpro system library",
        extension: ".libmol",
        comment: "!",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["libmol"],
        function: writers::libmol::write_libmol,
    },
    // bsedebug - unique extension .debug
    WriterEntry {
        name: "bsedebug",
        display: "BSE Debug",
        extension: ".debug",
        comment: "#",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["bsedebug", "debug"],
        function: writers::bsedebug::write_bsedebug,
    },
    // bsejson - aliases: json; extension .json (shared with qcschema)
    WriterEntry {
        name: "bsejson",
        display: "BSE JSON",
        extension: ".json",
        comment: "",
        valid: &["gto", "gto_cartesian", "gto_spherical", "scalar_ecp"],
        names: &["bsejson", "json"],
        function: writers::bsejson::write_bsejson,
    },
    // ricdwrap - extension .molcas (shared with molcas)
    WriterEntry {
        name: "ricdwrap",
        display: "acCD auxiliary basis wrapper",
        extension: ".molcas",
        comment: "*",
        valid: &["gto", "gto_cartesian", "gto_spherical"],
        names: &["ricdwrap"],
        function: writers::ricdwrap::write_ricdwrap,
    },
];

// Lazy-initialized writer map: name -> entry index.
// Maps all valid format names (canonical + aliases) to their entry index.
lazy_static::lazy_static! {
    static ref WRITER_MAP: HashMap<String, usize> = {
        let mut map = HashMap::new();
        for (idx, entry) in WRITER_ENTRIES.iter().enumerate() {
            for name in entry.names {
                map.insert(name.to_lowercase(), idx);
            }
        }
        map
    };

    // Extension -> format name map for auto-detection.
    // Some extensions are shared (e.g., .gbs, .json, .molcas, .bas).
    // For shared extensions, the first format encountered is used.
    static ref WRITER_EXTENSION_MAP: HashMap<String, &'static str> = {
        let mut map = HashMap::new();
        for entry in WRITER_ENTRIES.iter() {
            let ext = entry.extension.trim_start_matches('.').to_lowercase();
            // Only insert if not already present (keep first entry for shared extensions)
            map.entry(ext).or_insert(entry.name);
        }
        map
    };
}

/// Get writer entry by format name (canonical or alias).
fn get_writer_entry(fmt: &str) -> Option<&WriterEntry> {
    let fmt_lower = fmt.to_lowercase();
    WRITER_MAP.get(&fmt_lower).map(|idx| &WRITER_ENTRIES[*idx])
}

/// Get writer format by file extension.
///
/// Returns the canonical format name for a given file extension.
/// For shared extensions (e.g., .gbs, .json), the first format encountered
/// in WRITER_ENTRIES is returned.
///
/// # Arguments
///
/// * `ext` - The file extension (without leading dot, case insensitive)
///
/// # Returns
///
/// `Some(format_name)` if the extension is recognized, `None` otherwise.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// assert_eq!(get_writer_format_by_extension("nw"), Some("nwchem"));
/// assert_eq!(get_writer_format_by_extension("gbs"), Some("gaussian94"));
/// ```
pub fn get_writer_format_by_extension(ext: &str) -> Option<&'static str> {
    let ext_lower = ext.to_lowercase();
    WRITER_EXTENSION_MAP.get(&ext_lower).copied()
}

/// Get writer format information for a given format name.
///
/// This function handles format aliases (e.g., "g94" -> "gaussian94").
///
/// # Arguments
///
/// * `fmt` - The format name (case insensitive)
///
/// # Returns
///
/// `Some(WriterInfo)` if the format is valid, `None` otherwise.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let info = get_writer_info("nwchem").unwrap();
/// assert_eq!(info.display, "NWChem");
/// assert_eq!(info.extension, ".nw");
/// ```
pub fn get_writer_info(fmt: &str) -> Option<WriterInfo> {
    get_writer_entry(fmt).map(|e| WriterInfo {
        name: e.name,
        display: e.display,
        extension: e.extension,
        comment: e.comment,
        valid: e.valid,
        aliases: e.names,
    })
}

/// Return information about the basis set formats available for writing.
///
/// The returned data is a map of canonical format name to display name.
/// The format name can be passed as the `fmt` argument to
/// [`write_formatted_basis_str`].
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
    let ftypes: Option<HashSet<String>> = function_types.map(|ft| ft.into_iter().map(|s| s.to_lowercase()).collect());

    WRITER_ENTRIES
        .iter()
        .filter(|entry| {
            if let Some(ft) = &ftypes {
                let valid_types: HashSet<String> = entry.valid.iter().map(|s| s.to_string()).collect();
                ft.is_subset(&valid_types)
            } else {
                true
            }
        })
        .map(|entry| (entry.name.to_string(), entry.display.to_string()))
        .collect()
}

/// Return detailed information about writer formats including aliases.
///
/// The returned data is a map of canonical format name to (display name,
/// aliases).
pub fn get_writer_formats_with_aliases(function_types: Option<Vec<String>>) -> HashMap<String, (String, Vec<String>)> {
    let ftypes: Option<HashSet<String>> = function_types.map(|ft| ft.into_iter().map(|s| s.to_lowercase()).collect());

    WRITER_ENTRIES
        .iter()
        .filter(|entry| {
            if let Some(ft) = &ftypes {
                let valid_types: HashSet<String> = entry.valid.iter().map(|s| s.to_string()).collect();
                ft.is_subset(&valid_types)
            } else {
                true
            }
        })
        .map(|entry| {
            let aliases: Vec<String> =
                entry.names.iter().filter(|n| **n != entry.name).map(|n| n.to_string()).collect();
            (entry.name.to_string(), (entry.display.to_string(), aliases))
        })
        .collect()
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
    get_writer_entry(fmt)
        .map(|e| e.extension)
        .ok_or_else(|| BseError::ValueError(format!("Unknown basis set format '{}'", fmt)))
}

/// Returns the basis set data as a string representing the data in the
/// specified output format.
pub fn write_formatted_basis_str(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> String {
    write_formatted_basis_str_f(basis_dict, fmt, header).unwrap()
}

pub fn write_formatted_basis_str_f(basis_dict: &BseBasis, fmt: &str, header: Option<&str>) -> Result<String, BseError> {
    // make writers case insensitive
    let fmt_lower = fmt.to_lowercase();
    let entry =
        get_writer_entry(&fmt_lower).ok_or_else(|| BseError::ValueError(format!("Unknown writer format: {}", fmt)))?;

    // Determine if the writer supports all the types in the basis_dict
    if !entry.valid.is_empty() {
        let ftypes: HashSet<String> = basis_dict.function_types.iter().cloned().collect();
        let valid_types: HashSet<String> = entry.valid.iter().map(|s| s.to_string()).collect();
        if !ftypes.is_subset(&valid_types) {
            return bse_raise!(
                ValueError,
                "Writer {} does not support all function types: {:?} vs {:?}",
                entry.display,
                basis_dict.function_types,
                entry.valid
            );
        }
    }

    // Actually do the conversion
    let mut ret_str = (entry.function)(basis_dict);

    if let Some(header) = header {
        if !entry.comment.is_empty() {
            let comment_str = entry.comment;
            let header_str = header.split('\n').map(|line| format!("{comment_str}{line}")).join("\n");

            // HACK - Gaussian94Lib doesn't tolerate blank lines after the header
            if fmt_lower == "gaussian94lib" {
                ret_str.insert_str(0, &header_str);
            } else {
                ret_str.insert_str(0, &format!("{header_str}\n\n"));
            }
        }
    }

    // HACK - Psi4 requires the first non-comment line be spherical/cartesian, so we
    // have to add that before the header
    if fmt_lower == "psi4" {
        let types = &basis_dict.function_types;
        let harm_type = if types.contains(&"gto_cartesian".to_string()) { "cartesian" } else { "spherical" };
        ret_str.insert_str(0, &format!("{harm_type}\n\n"));
    }

    Ok(ret_str)
}
