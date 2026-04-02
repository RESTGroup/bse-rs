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

/// Obtain the name of an element given its Z (charge) number.
#[inline]
pub fn element_name_from_Z(z: i32) -> Option<&'static str> {
    ELEMENT_Z_MAP.get(&z).map(|(_, _, name)| *name)
}

/// Obtain the capitalized name of an element given its Z (charge) number.
#[inline]
pub fn element_name_from_Z_with_normalize(z: i32) -> Option<String> {
    ELEMENT_Z_MAP.get(&z).map(|(_, _, name)| {
        let mut name_normalized = name.to_string();
        if let Some(first_char) = name_normalized.chars().next() {
            name_normalized.replace_range(0..1, &first_char.to_uppercase().to_string());
        }
        name_normalized
    })
}

/// Obtain the Z (charge) number of an element given its name.
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
/// For example, converts \[0\] to 's' and \[0,1,2\] to 'spd'.
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
/// For case of sp shells (\[0,1\]), it returns 'l' instead of 'sp'. Otherwise,
/// it uses the `amint_to_char` function.
///
/// This convention happens to GAMESS and PQS.
#[inline]
pub fn amint_to_char_use_L(am: &[i32], hij: bool) -> String {
    if am == [0, 1] {
        "l".to_string()
    } else {
        amint_to_char(am, hij)
    }
}

/// Convert an angular momentum integer to a character.
///
/// The return value is a list of integers (to handle sp, spd, ... orbitals).
///
/// For example, converts 'p' to \[1\] and 'sp' to \[0,1\].
///
/// If hij is True, the ordering spdfghijkl is used. Otherwise, the ordering
/// will be spdfghikl (skipping j).
#[inline]
pub fn amchar_to_int(amchar: &str, hij: bool) -> Option<Vec<i32>> {
    let amchar_map = match hij {
        HIJ => AMCHAR_MAP_HIJ,
        HIK => AMCHAR_MAP_HIK,
    };
    amchar.to_lowercase().chars().map(|c| amchar_map.find(c).map(|i| i as i32)).collect()
}

/// Return the starting principal quantum numbers of electron shells
///
/// For example, an ECP covering 10 electrons will covers 1s, 2s, 2p shells. The
/// electrons shells will then start at 3s, 3p, 3d, and 4f (returned as [3, 3,
/// 3, 4])
///
/// If an ECP covers 30 electrons, then the shells will start at [5, 4, 4, 4]
///
/// Only fully-covered shells are counted. If a shell is partly covered, an
/// exception is raised.
///
/// The returned list is extended up to max_am.
///
/// Note: Since the main use of this is for ECPs, we only cover what can really
/// be found on the periodic table. No excited states!
///
/// # Arguments
/// * `nelectrons` - Number of electrons covered by an ECP
/// * `max_am` - Fill out the starting principal quantum numbers up to this am
///   (default: 20)
///
/// # Returns
/// The starting principal quantum numbers of s, p, d, and f shells.
pub fn electron_shells_start(nelectrons: i32, max_am: usize) -> Vec<i32> {
    if nelectrons < 0 {
        panic!("Can't have a negative number of electrons: {nelectrons}");
    }
    if nelectrons > 118 {
        panic!("Too many electrons for the periodic table: {nelectrons}");
    }

    // The usual filling order of electrons you learned in high school
    // Tuple of (am, nelec)
    let aminfo = [
        (0, 2), // He
        (0, 2),
        (1, 6), // Ne
        (0, 2),
        (1, 6), // Ar
        (0, 2),
        (2, 10),
        (1, 6), // Kr
        (0, 2),
        (2, 10),
        (1, 6), // Xe
        (0, 2),
        (3, 14),
        (2, 10),
        (1, 6), // Rn
        (0, 2),
        (3, 14),
        (2, 10),
        (1, 6), // Og
    ];

    // Special cases where ECPs don't follow the regular order
    let special_am = [
        (28, vec![0, 0, 0, 1, 1, 2]),
        (46, vec![0, 0, 0, 0, 1, 1, 1, 2, 2]),
        (60, vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 3]),
        (68, vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3]),
        (78, vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3]),
        (92, vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 3]),
    ]
    .into_iter()
    .collect::<std::collections::HashMap<i32, Vec<i32>>>();

    let contained_am = if let Some(am) = special_am.get(&nelectrons) {
        am.clone()
    } else {
        let mut remaining = nelectrons;
        let mut contained = Vec::new();

        for &(am, n) in &aminfo {
            if remaining >= n {
                remaining -= n;
                contained.push(am);
            } else {
                break;
            }
        }

        if remaining != 0 {
            panic!("Electrons cover a partial shell. {remaining} electrons left");
        }

        contained
    };

    let mut start = vec![
        contained_am.iter().filter(|&&am| am == 0).count() as i32 + 1,
        contained_am.iter().filter(|&&am| am == 1).count() as i32 + 2,
        contained_am.iter().filter(|&&am| am == 2).count() as i32 + 3,
        contained_am.iter().filter(|&&am| am == 3).count() as i32 + 4,
    ];

    // Extend with remaining AMs
    start.extend((5..=max_am as i32 + 1).collect::<Vec<_>>());

    start
}

#[inline]
pub fn function_type_from_am(shell_am: &[i32], base_type: &str, spherical_type: &str) -> String {
    if *shell_am.iter().max().unwrap() <= 1 {
        base_type.to_string()
    } else {
        format!("{base_type}_{spherical_type}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_z_map() {
        println!("{:?}", ELEMENT_Z_MAP[&118]);
    }
}
