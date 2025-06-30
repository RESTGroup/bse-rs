//! Main interface to Basis Set Exchange internal basis sets.
//!
//! This module contains the interface for getting basis set data and references
//! from the internal data store of basis sets.

use crate::prelude::*;

/// Obtain the version of the basis set exchange library (as a string).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub static MAIN_URL: &str = "https://www.basissetexchange.org";

/* #region data directory */

pub static BSE_DATA_DIR_SPECIFIED: Mutex<Option<String>> = Mutex::new(None);

/// Set the local directory for the basis set exchange data.
///
/// This directory is usually at `basis_set_exchange/data` of the repository
/// root of <https://github.com/MolSSI-BSE/basis_set_exchange>.
pub fn specify_bse_data_dir(dir: String) {
    let mut specified = BSE_DATA_DIR_SPECIFIED.lock().unwrap();
    *specified = Some(dir);
}

fn get_bse_data_dir_specified() -> String {
    let specified = BSE_DATA_DIR_SPECIFIED.lock().unwrap();
    specified.as_ref().unwrap_or(&String::new()).clone()
}

#[once]
fn get_bse_data_dir_env() -> String {
    std::env::var("BSE_DATA_DIR").unwrap_or_default()
}

#[once]
fn get_bse_data_dir_manifest() -> String {
    std::env::var("CARGO_MANIFEST_DIR")
        .map_or(String::new(), |dir| format!("{dir}/basis_set_exchange/basis_set_exchange/data"))
}

/// Get the available data directory for the basis set exchange library.
///
/// This function checks the following directories in order:
/// 1. The directory specified by `specify_bse_data_dir`.
/// 2. The directory specified by the environment variable `BSE_DATA_DIR`.
/// 3. The directory specified by the `CARGO_MANIFEST_DIR` (the directory where
///    your crate built).
pub fn get_bse_data_dir() -> Option<String> {
    let dir_specified = get_bse_data_dir_specified();
    let dir_env = get_bse_data_dir_env();
    let dir_manifest = get_bse_data_dir_manifest();

    // return the dir that `{dir}/METADATA.json` exists
    if std::path::Path::new(&format!("{dir_specified}/METADATA.json")).exists() {
        Some(dir_specified)
    } else if std::path::Path::new(&format!("{dir_env}/METADATA.json")).exists() {
        Some(dir_env)
    } else if std::path::Path::new(&format!("{dir_manifest}/METADATA.json")).exists() {
        Some(dir_manifest)
    } else {
        None
    }
}

/* #endregion */

/* #region read metadata */

/// Obtain the metadata for all basis sets.
///
/// The metadata includes information such as the display name of the basis set,
/// its versions, and what elements are included in the basis set.
///
/// The data is read from the METADATA.json file in the `data_dir` directory.
///
/// # Arguments
///
/// - `data_dir`: Data directory with all the basis set information. By default,
///   it is in the 'data' subdirectory of basis_set_exchange project.
pub fn get_metadata(data_dir: &str) -> HashMap<String, BseRootMetadata> {
    get_metadata_f(data_dir).unwrap()
}

#[cached(key = "String", convert = r#"{ data_dir.to_string() }"#)]
pub fn get_metadata_f(data_dir: &str) -> Result<HashMap<String, BseRootMetadata>, BseError> {
    let metadata_path = format!("{data_dir}/METADATA.json");
    let metadata = serde_json::from_str(&std::fs::read_to_string(metadata_path)?)?;
    Ok(metadata)
}

fn get_basis_metadata(name: &str, data_dir: &str) -> Result<BseRootMetadata, BseError> {
    // Transform the name into an internal representation
    let tr_name = misc::transform_basis_name(name);

    // Get the metadata for all basis sets
    let metadata = get_metadata_f(data_dir)?;

    if metadata.contains_key(&tr_name) {
        Ok(metadata[&tr_name].clone())
    } else {
        bse_raise!(
            DataNotFound,
            "Basis set `{name}` not found in metadata. Available basis sets: {:?}",
            metadata.keys().collect_vec()
        )?
    }
}

/* #endregion */

/* #region get_basis */

#[derive(Builder, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[builder(build_fn(error = "BseError"))]
#[serde(default)]
pub struct BseGetBasisArgs {
    #[builder(default)]
    pub elements: Option<String>,

    #[builder(default)]
    pub version: Option<String>,

    #[builder(default = false)]
    pub uncontract_general: bool,

    #[builder(default = false)]
    pub uncontract_spdf: bool,

    #[builder(default = false)]
    pub uncontract_segmented: bool,

    #[builder(default = false)]
    pub remove_free_primitives: bool,

    #[builder(default = false)]
    pub make_general: bool,

    #[builder(default = false)]
    pub optimize_general: bool,

    #[builder(default = 0)]
    pub augment_diffuse: i32,

    #[builder(default = 0)]
    pub augment_steep: i32,

    #[builder(default = 0)]
    pub get_aux: i32,

    #[builder(default)]
    pub data_dir: Option<String>,

    #[builder(default = true)]
    pub header: bool,
}

impl Default for BseGetBasisArgs {
    fn default() -> Self {
        BseGetBasisArgsBuilder::default().build().unwrap()
    }
}

pub fn get_basis(name: &str, args: BseGetBasisArgs) -> BseBasis {
    get_basis_f(name, args).unwrap()
}

pub fn get_basis_f(name: &str, args: BseGetBasisArgs) -> Result<BseBasis, BseError> {
    let data_dir = args.data_dir.clone().or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        )?;
    }
    let data_dir = data_dir.unwrap();

    let bs_data = get_basis_metadata(name, &data_dir)?;

    // If version is not specified, use the latest
    let ver = args.version.unwrap_or(bs_data.latest_version);
    if !bs_data.versions.contains_key(&ver) {
        bse_raise!(DataNotFound, "Version {ver} not found in metadata.")?;
    }

    // Compose the entire basis set (all elements)
    let table_relpath = &bs_data.versions[&ver].file_relpath;
    let mut basis_dict = compose::compose_table_basis_f(table_relpath, &data_dir)?;

    // Set the name (from the global metadata)
    // Only the list of all names will be returned from compose_table_basis
    basis_dict.name = bs_data.display_name.clone();

    // Handle optional arguments
    if let Some(elements) = &args.elements {
        // Convert to purely a list of strings that represent integers
        let elements = misc::expand_elements_f(elements)?;

        // Did the user pass an empty string or empty list?
        // If so, include all elements
        if !elements.is_empty() {
            let bs_elements = &basis_dict.elements;

            // Are elements part of this basis set?
            let bs_elements_keys = bs_elements.keys().map(|s| s.parse::<i32>().unwrap()).collect::<HashSet<_>>();
            let elements_set: HashSet<i32> = HashSet::from_iter(elements.clone());
            if !elements_set.is_subset(&bs_elements_keys) {
                bse_raise!(
                    DataNotFound,
                    "Elements {:?} not found in basis set `{name}`. Available elements: {:?}",
                    elements,
                    bs_elements_keys
                )?;
            }

            // Set to only the elements we want
            basis_dict.elements.retain(|el, _| elements_set.contains(&el.parse::<i32>().unwrap()));

            // Since we only grab some of the elements, we need to update the
            // function types used, too
            basis_dict.function_types = compose::whole_basis_types(&basis_dict.elements);
        }
    }

    // Note that from now on, the pipleline is going to modify basis_dict.
    // That is ok, since we are returned a unique instance from compose_table_basis.

    let mut needs_pruning = false;

    if args.remove_free_primitives {
        manip::remove_free_primitives(&mut basis_dict);
        needs_pruning = true;
    }

    if args.optimize_general {
        manip::optimize_general(&mut basis_dict);
        needs_pruning = true;
    }

    // uncontract_segmented implies uncontract_general
    if args.uncontract_segmented {
        manip::uncontract_segmented(&mut basis_dict);
        needs_pruning = true;
    }

    if args.uncontract_general {
        manip::uncontract_general(&mut basis_dict);
        needs_pruning = true;
    }

    if args.uncontract_spdf {
        manip::uncontract_spdf(&mut basis_dict, 0);
        needs_pruning = true;
    }

    if args.make_general {
        manip::make_general(&mut basis_dict, false);
        needs_pruning = true;
    }

    if needs_pruning {
        manip::prune_basis(&mut basis_dict);
    }

    if args.augment_diffuse > 0 {
        manip::geometric_augmentation(&mut basis_dict, args.augment_diffuse, false);
    }

    Ok(basis_dict)
}

/* #endregion */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_basis() {
        let data_dir = get_bse_data_dir().expect("Data directory not found");
        let args = BseGetBasisArgsBuilder::default().data_dir(Some(data_dir.clone())).build().unwrap();
        let basis = get_basis_f("sto-3g", args).expect("Failed to get basis set");
        assert_eq!(basis.name, "STO-3G");
        let basis_json = serde_json::to_string_pretty(&basis).expect("Failed to serialize basis set to JSON");
        println!("Basis set JSON: {basis_json}");
    }
}
