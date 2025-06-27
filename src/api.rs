//! Main interface to Basis Set Exchange internal basis sets.
//!
//! This module contains the interface for getting basis set data and references from the internal
//! data store of basis sets.

use crate::{error::BseError, prelude::*};

/// Obtain the version of the basis set exchange library (as a string).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub static MAIN_URL: &str = "https://www.basissetexchange.org";

/* #region data directory */

pub static BSE_DATA_DIR_SPECIFIED: Mutex<Option<String>> = Mutex::new(None);

/// Set the local directory for the basis set exchange data.
///
/// This directory is usually at `basis_set_exchange/data` of the repository root of
/// <https://github.com/MolSSI-BSE/basis_set_exchange>.
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
/// 3. The directory specified by the `CARGO_MANIFEST_DIR` (the directory where your crate built).
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
/// The metadata includes information such as the display name of the basis set, its versions, and
/// what elements are included in the basis set.
///
/// The data is read from the METADATA.json file in the `data_dir` directory.
///
/// # Arguments
///
/// - `data_dir`: Data directory with all the basis set information. By default, it is in the 'data'
///   subdirectory of basis_set_exchange project.
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
    elements: Option<String>,

    #[builder(default)]
    version: Option<String>,

    #[builder(default = false)]
    uncontract_general: bool,

    #[builder(default = false)]
    uncontract_spdf: bool,

    #[builder(default = false)]
    uncontract_segmented: bool,

    #[builder(default = false)]
    remove_free_primitives: bool,

    #[builder(default = false)]
    make_general: bool,

    #[builder(default = false)]
    optimize_general: bool,

    #[builder(default = 0)]
    augment_diffuse: i32,

    #[builder(default = 0)]
    augment_steep: i32,

    #[builder(default = 0)]
    get_aux: i32,

    #[builder(default)]
    data_dir: Option<String>,

    #[builder(default = true)]
    header: bool,
}

impl Default for BseGetBasisArgs {
    fn default() -> Self {
        BseGetBasisArgsBuilder::default().build().unwrap()
    }
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

    let ver = match args.version {
        Some(v) => v,
        None => bs_data.latest_version,
    };
    if !bs_data.versions.contains_key(&ver) {
        bse_raise!(
            DataNotFound,
            "Version {ver} not found in metadata. Available versions: {:?}",
            bs_data.versions.keys().collect_vec()
        )?;
    }
    let table_relpath = &bs_data.versions[&ver].file_relpath;
    let mut basis_dict = compose::compose_table_basis_f(table_relpath, &data_dir)?;
    basis_dict.name = bs_data.display_name.clone();

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
