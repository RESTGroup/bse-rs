//! Read a basis set file in a given format.

use crate::prelude::*;

/// Format information for a reader.
#[allow(dead_code)]
struct ReaderFormat {
    display: &'static str,
    extension: &'static str,
}

fn reader_format_map() -> HashMap<&'static str, ReaderFormat> {
    HashMap::from([
        ("nwchem", ReaderFormat { display: "NWChem", extension: ".nw" }),
        ("gaussian94", ReaderFormat { display: "Gaussian94", extension: ".gbs" }),
        ("g94", ReaderFormat { display: "Gaussian94", extension: ".gbs" }),
        ("turbomole", ReaderFormat { display: "Turbomole", extension: ".tm" }),
        ("dalton", ReaderFormat { display: "Dalton", extension: ".mol" }),
        ("molcas", ReaderFormat { display: "Molcas", extension: ".molcas" }),
        ("molpro", ReaderFormat { display: "Molpro", extension: ".mpro" }),
        ("genbas", ReaderFormat { display: "Genbas", extension: ".genbas" }),
        ("cfour", ReaderFormat { display: "CFOUR", extension: ".c4bas" }),
        ("aces2", ReaderFormat { display: "ACESII", extension: ".aces2" }),
        ("gamess_us", ReaderFormat { display: "GAMESS US", extension: ".gms" }),
        ("cp2k", ReaderFormat { display: "CP2K", extension: ".cp2k" }),
        ("crystal", ReaderFormat { display: "Crystal", extension: ".crystal" }),
    ])
}

#[allow(dead_code)]
struct Reader {
    display: &'static str,
    extension: &'static str,
    function: fn(&str) -> Result<BseBasisMinimal, BseError>,
}

fn reader_map(fmt: &str) -> Option<Reader> {
    match fmt {
        "nwchem" => Some(Reader { display: "NWChem", extension: ".nw", function: readers::nwchem::read_nwchem }),
        "gaussian94" | "g94" => {
            Some(Reader { display: "Gaussian94", extension: ".gbs", function: readers::g94::read_g94 })
        },
        "turbomole" => {
            Some(Reader { display: "Turbomole", extension: ".tm", function: readers::turbomole::read_turbomole })
        },
        "dalton" => Some(Reader { display: "Dalton", extension: ".mol", function: readers::dalton::read_dalton }),
        "molcas" => Some(Reader { display: "Molcas", extension: ".molcas", function: readers::molcas::read_molcas }),
        "molpro" => Some(Reader { display: "Molpro", extension: ".mpro", function: readers::molpro::read_molpro }),
        "genbas" | "cfour" | "aces2" => {
            Some(Reader { display: "Genbas", extension: ".genbas", function: readers::genbas::read_genbas })
        },
        "gamess_us" => {
            Some(Reader { display: "GAMESS US", extension: ".gms", function: readers::gamess_us::read_gamess_us })
        },
        "cp2k" => Some(Reader { display: "CP2K", extension: ".cp2k", function: readers::cp2k::read_cp2k }),
        "crystal" => {
            Some(Reader { display: "Crystal", extension: ".crystal", function: readers::crystal::read_crystal })
        },
        _ => None,
    }
}

/// Return information about the basis set formats available for reading.
///
/// The returned data is a map of format name to display name. The format
/// name can be passed as the `fmt` argument to [`read_formatted_basis_str`].
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
    let format_map = reader_format_map();
    format_map.into_iter().map(|(k, v)| (k.to_string(), v.display.to_string())).collect()
}

pub fn read_formatted_basis_str(basis_str: &str, fmt: &str) -> BseBasisMinimal {
    read_formatted_basis_str_f(basis_str, fmt).unwrap()
}

pub fn read_formatted_basis_str_f(basis_str: &str, fmt: &str) -> Result<BseBasisMinimal, BseError> {
    let fmt = fmt.to_lowercase();
    let reader = reader_map(&fmt).map_or(bse_raise!(ValueError, "Unknown reader format: {fmt}"), Ok)?;
    (reader.function)(basis_str)
}
