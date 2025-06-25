//! Main interface to Basis Set Exchange internal basis sets.
//!
//! This module contains the interface for getting basis set data and references from the internal
//! data store of basis sets.

use crate::{error::BseError, prelude::*};

/// Obtain the version of the basis set exchange library (as a string).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

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
#[cached(key = "String", convert = "{data_dir.to_string()}")]
pub fn get_metadata(data_dir: &str) -> BseRootMetadata {
    get_metadata_f(data_dir).unwrap()
}

#[cached(key = "String", convert = "{data_dir.to_string()}")]
pub fn get_metadata_f(data_dir: &str) -> Result<BseRootMetadata, BseError> {
    let metadata_path = format!("{data_dir}/METADATA.json");
    let metadata: BseRootMetadata = serde_json::from_str(&std::fs::read_to_string(metadata_path)?)?;
    Ok(metadata)
}
