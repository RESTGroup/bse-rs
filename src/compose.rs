//! Functions related to composing basis sets from individual components.

use crate::prelude::*;

/// Creates a 'table' basis from an table json file.
///
/// This function reads the info from the given file, and reads all the
/// elemental basis set information from the files listed therein. It then
/// composes all the information together into one 'table' basis dictionary
///
/// Note that the data returned from this function will not be shared, even if
/// the function is called again with the same arguments.
pub fn compose_table_basis(file_relpath: &str, data_dir: &str) -> BseBasis {
    compose_table_basis_f(file_relpath, data_dir).unwrap()
}

/// Creates an 'elemental' basis from an elemental json file.
///
/// This function reads the info from the given file, and reads all the
/// component basis set information from the files listed therein. It then
/// composes all the information together into one 'elemental' basis dictionary
pub fn compose_elemental_basis(file_relpath: &str, data_dir: &str) -> HashMap<String, BseBasisElement> {
    compose_elemental_basis_f(file_relpath, data_dir).unwrap()
}

pub(crate) fn whole_basis_types(basis_elements: &HashMap<String, BseBasisElement>) -> Vec<String> {
    let mut function_types = HashSet::new();
    for basis_element in basis_elements.values() {
        if let Some(shells) = &basis_element.electron_shells {
            for shell in shells {
                function_types.insert(shell.function_type.clone());
            }
        }
        if basis_element.ecp_potentials.is_some() {
            function_types.insert("scalar_ecp".to_string());
        }
    }
    function_types.into_iter().sorted().collect()
}

pub fn compose_elemental_basis_f(
    file_relpath: &str,
    data_dir: &str,
) -> Result<HashMap<String, BseBasisElement>, BseError> {
    // read skeleton element
    let skel_element_relpath = file_relpath;
    let skel_element = fields::read_skel_element_file_f(skel_element_relpath, data_dir)?;

    let mut skel_component_set = HashSet::new();
    for skel_component_realpaths in skel_element.elements.values() {
        for skel_component_realpath in skel_component_realpaths.components.iter() {
            skel_component_set.insert(skel_component_realpath.clone());
        }
    }

    enum BseSkelComponent {
        Gto(BseSkelComponentGto),
        Ecp(BseSkelComponentEcp),
    }

    let mut skel_component_map: HashMap<String, BseSkelComponent> = HashMap::new();
    for skel_component_realpath in skel_component_set {
        // read skeleton component, to get the components
        if skel_component_map.contains_key(&skel_component_realpath) {
            continue; // already read
        }
        if let Ok(skel_component) = fields::read_skel_component_gto_file_f(&skel_component_realpath, data_dir) {
            skel_component_map.insert(skel_component_realpath, BseSkelComponent::Gto(skel_component));
        } else if let Ok(skel_component) = fields::read_skel_component_ecp_file_f(&skel_component_realpath, data_dir) {
            skel_component_map.insert(skel_component_realpath, BseSkelComponent::Ecp(skel_component));
        } else {
            bse_raise!(DataError, "Internal bug: read skeleton component json failed {skel_component_realpath}")?;
        }
    }

    // compose basis elements from skeleton element
    let mut basis_elements = HashMap::new();
    for (atomic_number, skel_element) in &skel_element.elements {
        let mut basis_element = BseBasisElement::default();

        for component_relpath in &skel_element.components {
            match &skel_component_map[component_relpath] {
                BseSkelComponent::Gto(gto_component) => {
                    // read skeleton component (gto) and parse
                    let gto_element = &gto_component.elements[atomic_number];
                    let gto_reference = BseBasisReference {
                        reference_keys: gto_element.references.clone(),
                        reference_description: gto_component.description.clone(),
                    };
                    basis_element.references.push(gto_reference);
                    if basis_element.electron_shells.is_none() {
                        basis_element.electron_shells = Some(gto_element.electron_shells.clone());
                    } else if let Some(shells) = &mut basis_element.electron_shells {
                        shells.extend_from_slice(&gto_element.electron_shells);
                    }
                },
                BseSkelComponent::Ecp(ecp_component) => {
                    // read skeleton component (ecp) and parse
                    let ecp_element = &ecp_component.elements[atomic_number];
                    let ecp_reference = BseBasisReference {
                        reference_keys: ecp_element.references.clone(),
                        reference_description: ecp_component.description.clone(),
                    };
                    basis_element.references.push(ecp_reference);
                    if basis_element.ecp_potentials.is_none() {
                        basis_element.ecp_potentials = Some(ecp_element.ecp_potentials.clone());
                    } else if let Some(potentials) = &mut basis_element.ecp_potentials {
                        potentials.extend_from_slice(&ecp_element.ecp_potentials);
                    }
                    if basis_element.ecp_electrons.is_none() {
                        basis_element.ecp_electrons = Some(ecp_element.ecp_electrons);
                    } else if let Some(electrons) = basis_element.ecp_electrons
                        && electrons != ecp_element.ecp_electrons
                    {
                        bse_raise!(
                            DataError,
                            "Internal bug: ECP electron not match ({} != {})",
                            electrons,
                            ecp_element.ecp_electrons
                        )?;
                    }
                },
            }
        }
        basis_elements.insert(atomic_number.clone(), basis_element);
    }

    Ok(basis_elements)
}

#[cached(key = "String", convert = r#"{ format!("{data_dir}/{file_relpath}") }"#)]
pub fn compose_table_basis_f(file_relpath: &str, data_dir: &str) -> Result<BseBasis, BseError> {
    // read skeleton table
    let skel_table_relpath = file_relpath;
    let skel_table = fields::read_skel_table_file_f(skel_table_relpath, data_dir)?;

    // read skeleton elements and maps
    let mut skel_element_map = HashMap::new();
    for skel_element_realpath in skel_table.elements.values() {
        // read skeleton element, to get the components
        if skel_element_map.contains_key(skel_element_realpath) {
            continue; // already read
        }
        let skel_element = compose_elemental_basis_f(skel_element_realpath, data_dir)?;
        skel_element_map.insert(skel_element_realpath.clone(), skel_element);
    }

    // generate field elements in BseBasis
    let mut basis_elements = HashMap::new();
    for (atomic_number, skl_element_realpath) in &skel_table.elements {
        basis_elements.insert(atomic_number.clone(), skel_element_map[skl_element_realpath][atomic_number].clone());
    }

    // version is also defined in metadata (field version), but can be inferred from
    // the file name. take version `1` from file `def2-TZVP.1.table.json`, which
    // is the last third value. if version is not found, use -1 as default.
    let version = catch_unwind(|| skel_table_relpath.split('.').rev().collect_vec()[2].to_string()).map_err(|_| {
        BseError::DataError(format!(
            "{}BseError::DataError: Version annotation not found in {skel_table_relpath}",
            bse_trace!()
        ))
    })?;

    // function types are obtained by iterating elements.
    let function_types = whole_basis_types(&basis_elements);

    // read skeleton metadata
    let name_prefix = skel_table_relpath.split('.').next().unwrap_or("");
    let metadata_relpath = format!("{name_prefix}.metadata.json");
    let skel_metadata = fields::read_skel_metadata_file_f(&metadata_relpath, data_dir)?;

    // temporarily use the first name in metadata.names
    let name = skel_metadata.names.first().cloned().unwrap();

    let basis = BseBasis {
        molssi_bse_schema: BseMolssiBseSchema { schema_type: "complete".into(), schema_version: "0.1".into() },
        revision_description: skel_table.revision_description,
        revision_date: skel_table.revision_date,
        version,
        function_types,
        name,
        names: skel_metadata.names,
        tags: skel_metadata.tags,
        family: skel_metadata.family,
        description: skel_metadata.description,
        role: skel_metadata.role,
        auxiliaries: skel_metadata.auxiliaries,
        elements: basis_elements,
    };
    Ok(basis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_table_basis() {
        let data_dir = get_bse_data_dir().unwrap();

        let time = std::time::Instant::now();
        let table_relpath = "def2-TZVP.1.table.json";
        let basis = compose_table_basis(table_relpath, &data_dir);
        let json_str = serde_json::to_string_pretty(&basis.elements["1"]).unwrap();
        println!("{}", &json_str);
        let json_str = serde_json::to_string_pretty(&basis.elements["54"]).unwrap();
        println!("{}", &json_str);
        println!("compose_table_basis took {} ms", time.elapsed().as_millis());

        let time = std::time::Instant::now();
        let table_relpath = "def2-TZVP.1.table.json";
        let _basis = compose_table_basis(table_relpath, &data_dir);
        println!("compose_table_basis took {} ms", time.elapsed().as_millis());
    }
}
