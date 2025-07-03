//! Read a basis set file in a given format.

use crate::prelude::*;

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
        _ => None,
    }
}

pub fn read_formatted_basis_str(basis_str: &str, fmt: &str) -> BseBasisMinimal {
    read_formatted_basis_str_f(basis_str, fmt).unwrap()
}

pub fn read_formatted_basis_str_f(basis_str: &str, fmt: &str) -> Result<BseBasisMinimal, BseError> {
    let fmt = fmt.to_lowercase();
    let reader = reader_map(&fmt).map_or(bse_raise!(ValueError, "Unknown format: {fmt}"), Ok)?;
    (reader.function)(basis_str)
}
