//! Functions for looking up element names, numbers, and symbols and for
//! converting between angular momentum conventions.
//!
//! This module has functions for looking up element information by symbol,
//! name, or number. It also has functions for converting angular momentum
//! between integers (0, 1, 2) and letters (s, p, d).

pub use crate::lut_data::*;
use crate::prelude::*;

/// Obtain a list of the names of all the elements.
#[once]
pub fn all_element_names() -> Vec<String> {
    ELEMENT_NAME_MAP.keys().map(|s| s.to_string()).collect()
}

/// Obtain elemental data given a Z (charge) number.
#[inline]
pub fn element_data_from_Z(z: i32) -> Option<(&'static str, i32, &'static str)> {
    ELEMENT_Z_MAP.get(&z).copied()
}

/// Obtain elemental data given a symbol.
#[inline]
pub fn element_data_from_sym(sym: &str) -> Option<(&'static str, i32, &'static str)> {
    let sym_lower = sym.to_lowercase();
    ELEMENT_SYM_MAP.get(&sym_lower).copied()
}

/// Obtain elemental data given an elemental name.
#[inline]
pub fn element_data_from_name(name: &str) -> Option<(&'static str, i32, &'static str)> {
    let name_lower = name.to_lowercase();
    ELEMENT_NAME_MAP.get(&name_lower).copied()
}

/// Obtain the symbol of an element given its Z (charge) number.
#[inline]
pub fn element_name_from_Z(z: i32) -> Option<&'static str> {
    ELEMENT_Z_MAP.get(&z).map(|(_, _, name)| *name)
}

/// Obtain the symbol of an element given its symbol.
#[inline]
pub fn element_Z_from_name(name: &str) -> Option<i32> {
    let name_lower = name.to_lowercase();
    ELEMENT_NAME_MAP.get(&name_lower).map(|(_, z, _)| *z)
}

/// Obtain the symbol of an element given its Z (charge) number.
#[inline]
pub fn element_sym_from_Z(z: i32) -> Option<&'static str> {
    ELEMENT_Z_MAP.get(&z).map(|(sym, _, _)| *sym)
}

/// Obtain the symbol of an element given its Z (charge) number, where the first
/// character is captalized.
#[inline]
pub fn element_sym_from_Z_with_normalize(z: i32) -> Option<String> {
    ELEMENT_Z_MAP.get(&z).map(|(sym, _, _)| {
        let mut sym_normalized = sym.to_string();
        if let Some(first_char) = sym_normalized.chars().next() {
            sym_normalized.replace_range(0..1, &first_char.to_uppercase().to_string());
        }
        sym_normalized
    })
}

/// Obtain the Z (charge) number of an element given its symbol.
#[inline]
pub fn element_Z_from_sym(sym: &str) -> Option<i32> {
    let sym_lower = sym.to_lowercase();
    ELEMENT_SYM_MAP.get(&sym_lower).map(|(_, z, _)| *z)
}

/// Obtain the Z (charge) number of an element given its name.
#[inline]
pub fn element_Z_from_str(s: &str) -> Option<i32> {
    if let Ok(z) = s.parse::<i32>() {
        return Some(z);
    } else if let Some(data) = element_data_from_sym(s) {
        return Some(data.1);
    } else if let Some(data) = element_data_from_name(s) {
        return Some(data.1);
    }
    None
}

/// Convert an angular momentum integer to a character.
///
/// The input is a list (to handle sp, spd, ... orbitals). The return value is a
/// string.
///
/// For example, converts [0] to 's' and [0,1,2] to 'spd'.
///
/// If hij is True, the ordering spdfghijkl is used. Otherwise, the ordering
/// will be spdfghikl (skipping j).
#[inline]
pub fn amint_to_char(am: &[i32], hij: bool) -> String {
    let amchar_map = match hij {
        HIJ => AMCHAR_MAP_HIJ,
        HIK => AMCHAR_MAP_HIK,
    };
    am.iter().map(|&a| amchar_map.chars().nth(a as usize).unwrap()).collect()
}

/// Convert an angular momentum integer to a character.
///
/// For case of sp shells ([0,1]), it returns 'l' instead of 'sp'. Otherwise, it
/// uses the `amint_to_char` function.
///
/// This convention happens to GAMESS and PQS.
#[inline]
pub fn amint_to_char_use_L(am: &[i32], hij: bool) -> String {
    if am == [0, 1] { "l".to_string() } else { amint_to_char(am, hij) }
}

/// Convert an angular momentum integer to a character.
///
/// The return value is a list of integers (to handle sp, spd, ... orbitals).
///
/// For example, converts 'p' to [1] and 'sp' to [0,1].
///
/// If hij is True, the ordering spdfghijkl is used. Otherwise, the ordering
/// will be spdfghikl (skipping j).
#[inline]
pub fn amchar_to_int(amchar: &str, hij: bool) -> Option<Vec<i32>> {
    let amchar_map = match hij {
        HIJ => AMCHAR_MAP_HIJ,
        HIK => AMCHAR_MAP_HIK,
    };
    amchar.chars().map(|c| amchar_map.find(c).map(|i| i as i32)).collect()
}

#[inline]
pub fn function_type_from_am(shell_am: &[i32], base_type: &str, spherical_type: &str) -> String {
    if *shell_am.iter().max().unwrap() <= 1 { base_type.to_string() } else { format!("{base_type}_{spherical_type}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_z_map() {
        println!("{:?}", ELEMENT_Z_MAP[&118]);
    }
}
