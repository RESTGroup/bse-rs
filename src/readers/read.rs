//! Read a basis set file in a given format.

use crate::prelude::*;

/// Reader format information.
#[derive(Clone, Copy)]
pub struct ReaderInfo {
    /// Canonical format name
    pub name: &'static str,
    /// Display name for the format
    pub display: &'static str,
    /// File extension for the format
    pub extension: &'static str,
    /// Aliases for this format (including extension if unique)
    pub aliases: &'static [&'static str],
}

impl ReaderInfo {
    /// Get the file extension without the leading dot.
    pub fn extension_without_dot(&self) -> &'static str {
        self.extension.trim_start_matches('.')
    }

    /// Check if a given name is an alias for this format.
    pub fn is_alias(&self, name: &str) -> bool {
        self.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
    }
}

/// Reader entry with all format information.
#[derive(Clone, Copy)]
struct ReaderEntry {
    /// Canonical format name
    name: &'static str,
    /// Display name for the format
    display: &'static str,
    /// File extension for the format
    extension: &'static str,
    /// All valid names for this format (canonical + aliases)
    names: &'static [&'static str],
    /// Reader function
    function: fn(&str) -> Result<BseBasisMinimal, BseError>,
}

/// All reader format entries.
const READER_ENTRIES: &[ReaderEntry] = &[
    // nwchem - unique extension .nw
    ReaderEntry {
        name: "nwchem",
        display: "NWChem",
        extension: ".nw",
        names: &["nwchem", "nw"],
        function: readers::nwchem::read_nwchem,
    },
    // gaussian94 - extension .gbs unique for reader
    ReaderEntry {
        name: "gaussian94",
        display: "Gaussian94",
        extension: ".gbs",
        names: &["gaussian94", "g94", "gbs"],
        function: readers::g94::read_g94,
    },
    // turbomole - unique extension .tm
    ReaderEntry {
        name: "turbomole",
        display: "Turbomole",
        extension: ".tm",
        names: &["turbomole", "tm"],
        function: readers::turbomole::read_turbomole,
    },
    // dalton - unique extension .mol
    ReaderEntry {
        name: "dalton",
        display: "Dalton",
        extension: ".mol",
        names: &["dalton", "mol"],
        function: readers::dalton::read_dalton,
    },
    // molcas - unique extension .molcas
    ReaderEntry {
        name: "molcas",
        display: "Molcas",
        extension: ".molcas",
        names: &["molcas"],
        function: readers::molcas::read_molcas,
    },
    // molpro - unique extension .mpro
    ReaderEntry {
        name: "molpro",
        display: "Molpro",
        extension: ".mpro",
        names: &["molpro", "mpro"],
        function: readers::molpro::read_molpro,
    },
    // genbas - aliases: cfour, aces2; extension .genbas
    ReaderEntry {
        name: "genbas",
        display: "Genbas",
        extension: ".genbas",
        names: &["genbas", "cfour", "aces2", "genbas"],
        function: readers::genbas::read_genbas,
    },
    // gamess_us - unique extension .gms
    ReaderEntry {
        name: "gamess_us",
        display: "GAMESS US",
        extension: ".gms",
        names: &["gamess_us", "gms"],
        function: readers::gamess_us::read_gamess_us,
    },
    // cp2k - unique extension .cp2k
    ReaderEntry {
        name: "cp2k",
        display: "CP2K",
        extension: ".cp2k",
        names: &["cp2k"],
        function: readers::cp2k::read_cp2k,
    },
    // crystal - unique extension .crystal
    ReaderEntry {
        name: "crystal",
        display: "Crystal",
        extension: ".crystal",
        names: &["crystal"],
        function: readers::crystal::read_crystal,
    },
    // libmol - unique extension .libmol
    ReaderEntry {
        name: "libmol",
        display: "Libmol",
        extension: ".libmol",
        names: &["libmol"],
        function: readers::libmol::read_libmol,
    },
    // gbasis - unique extension .gbas
    ReaderEntry {
        name: "gbasis",
        display: "GBasis",
        extension: ".gbas",
        names: &["gbasis", "gbas"],
        function: readers::gbasis::read_gbasis,
    },
    // demon2k - unique extension .dmon
    ReaderEntry {
        name: "demon2k",
        display: "deMon2k",
        extension: ".dmon",
        names: &["demon2k", "dmon"],
        function: readers::demon2k::read_demon2k,
    },
    // veloxchem - unique extension .vlx
    ReaderEntry {
        name: "veloxchem",
        display: "VeloxChem",
        extension: ".vlx",
        names: &["veloxchem", "vlx"],
        function: readers::veloxchem::read_veloxchem,
    },
    // ricdlib - unique extension .ricd
    ReaderEntry {
        name: "ricdlib",
        display: "RICDlib",
        extension: ".ricd",
        names: &["ricdlib", "ricd"],
        function: readers::ricdlib::read_ricdlib,
    },
    // json - aliases: bsejson
    ReaderEntry {
        name: "json",
        display: "JSON",
        extension: ".json",
        names: &["json", "bsejson"],
        function: readers::bsejson::read_bsejson,
    },
];

// Lazy-initialized reader map: name -> entry index.
// Maps all valid format names (canonical + aliases) to their entry index.
lazy_static::lazy_static! {
    static ref READER_MAP: HashMap<String, usize> = {
        let mut map = HashMap::new();
        for (idx, entry) in READER_ENTRIES.iter().enumerate() {
            for name in entry.names {
                map.insert(name.to_lowercase(), idx);
            }
        }
        map
    };

    // Extension -> format name map for auto-detection.
    // Extensions are unique for readers, so this is straightforward.
    static ref READER_EXTENSION_MAP: HashMap<String, &'static str> = {
        let mut map = HashMap::new();
        for entry in READER_ENTRIES.iter() {
            let ext = entry.extension.trim_start_matches('.').to_lowercase();
            map.insert(ext, entry.name);
        }
        map
    };
}

/// Get reader entry by format name (canonical or alias).
fn get_reader_entry(fmt: &str) -> Option<&ReaderEntry> {
    let fmt_lower = fmt.to_lowercase();
    READER_MAP.get(&fmt_lower).map(|idx| &READER_ENTRIES[*idx])
}

/// Get reader format by file extension.
///
/// Returns the canonical format name for a given file extension.
/// For readers, all extensions are unique.
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
/// assert_eq!(get_reader_format_by_extension("nw"), Some("nwchem"));
/// assert_eq!(get_reader_format_by_extension("gbs"), Some("gaussian94"));
/// ```
pub fn get_reader_format_by_extension(ext: &str) -> Option<&'static str> {
    let ext_lower = ext.to_lowercase();
    READER_EXTENSION_MAP.get(&ext_lower).copied()
}

/// Get reader format information for a given format name.
///
/// This function handles format aliases (e.g., "g94" -> "gaussian94").
///
/// # Arguments
///
/// * `fmt` - The format name (case insensitive)
///
/// # Returns
///
/// `Some(ReaderInfo)` if the format is valid, `None` otherwise.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let info = get_reader_info("nwchem").unwrap();
/// assert_eq!(info.display, "NWChem");
/// assert_eq!(info.extension, ".nw");
/// ```
pub fn get_reader_info(fmt: &str) -> Option<ReaderInfo> {
    get_reader_entry(fmt).map(|e| ReaderInfo {
        name: e.name,
        display: e.display,
        extension: e.extension,
        aliases: e.names,
    })
}

/// Return information about the basis set formats available for reading.
///
/// The returned data is a map of canonical format name to display name.
/// The format name can be passed as the `fmt` argument to
/// [`read_formatted_basis_str`].
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let formats = get_reader_formats();
/// assert!(!formats.is_empty());
/// assert!(formats.contains_key("nwchem"));
/// println!("Available reader formats: {:?}", formats);
/// ```
pub fn get_reader_formats() -> HashMap<String, String> {
    READER_ENTRIES.iter().map(|entry| (entry.name.to_string(), entry.display.to_string())).collect()
}

/// Return detailed information about reader formats including aliases.
///
/// The returned data is a map of canonical format name to (display name,
/// aliases).
pub fn get_reader_formats_with_aliases() -> HashMap<String, (String, Vec<String>)> {
    READER_ENTRIES
        .iter()
        .map(|entry| {
            let aliases: Vec<String> =
                entry.names.iter().filter(|n| **n != entry.name).map(|n| n.to_string()).collect();
            (entry.name.to_string(), (entry.display.to_string(), aliases))
        })
        .collect()
}

pub fn read_formatted_basis_str(basis_str: &str, fmt: &str) -> BseBasisMinimal {
    read_formatted_basis_str_f(basis_str, fmt).unwrap()
}

pub fn read_formatted_basis_str_f(basis_str: &str, fmt: &str) -> Result<BseBasisMinimal, BseError> {
    let entry = get_reader_entry(fmt).ok_or_else(|| BseError::ValueError(format!("Unknown reader format: {}", fmt)))?;
    (entry.function)(basis_str)
}
