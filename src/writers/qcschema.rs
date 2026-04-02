//! Conversion of basis sets to QCSchema JSON format

use crate::prelude::*;
use serde_json::{json, to_string_pretty};

/// Converts a basis set to QCSchema JSON
///
/// Note that the output is a string
pub fn write_qcschema(basis: &BseBasis) -> String {
    // Uncontract all but SP
    let mut basis = basis.clone();
    manip::uncontract_spdf(&mut basis, 1);
    sort::sort_basis(&mut basis);

    let basis_name = &basis.name;
    let basis_desc = &basis.description;
    let mut new_basis = json!({
        "schema_name": "qcschema_basis",
        "schema_version": 1,
        "name": basis_name,
        "description": basis_desc
    });

    // For the 'center_data' key in the schema
    let mut center_data = serde_json::Map::new();

    for (el, eldata) in &basis.elements {
        let element_sym = lut::element_sym_from_Z(el.parse().unwrap()).unwrap();
        let entry_name = format!("{element_sym}_{basis_name}");

        let mut eldata_json = serde_json::to_value(eldata).unwrap();
        let eldata_obj = eldata_json.as_object_mut().unwrap();
        eldata_obj.remove("references");

        if let Some(shells) = eldata_obj.get_mut("electron_shells") {
            if let Some(shells_arr) = shells.as_array_mut() {
                for shell in shells_arr {
                    let shell_obj = shell.as_object_mut().unwrap();
                    shell_obj.remove("region");
                    let func = shell_obj.remove("function_type").unwrap().as_str().unwrap().to_string();

                    let harmonic_type = if func == "gto_spherical" {
                        "spherical"
                    } else {
                        // Set to cartesian if explicitly cartesian, or if it is an
                        // s or p shell
                        "cartesian"
                    };
                    shell_obj.insert("harmonic_type".to_string(), json!(harmonic_type));
                }
            }
        }

        if eldata_obj.contains_key("ecp_electrons") {
            if let Some(pots) = eldata_obj.get_mut("ecp_potentials") {
                if let Some(pots_arr) = pots.as_array_mut() {
                    for pot in pots_arr {
                        let pot_obj = pot.as_object_mut().unwrap();
                        let ecp_type = pot_obj["ecp_type"].as_str().unwrap().replace("_ecp", "");
                        pot_obj.insert("ecp_type".to_string(), json!(ecp_type));
                    }
                }
            }
        }

        center_data.insert(entry_name, eldata_json);
    }

    new_basis["center_data"] = json!(center_data);

    to_string_pretty(&new_basis).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_qcschema() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_qcschema(&basis);
        println!("{output}");
    }
}
