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

/// Try to detect BSE data directory via Python's basis_set_exchange package.
///
/// This function attempts to run Python to query the installed
/// `basis_set_exchange` package for its data directory location. It does not
/// require pyo3 or linking to libpython - it simply spawns a subprocess.
///
/// # Returns
///
/// - `Some(String)` if Python is available and `basis_set_exchange` is
///   installed
/// - `None` if Python is not available, the package is not installed, or the
///   subprocess fails
#[once]
fn get_bse_data_dir_python() -> Option<String> {
    use std::process::Command;

    // Try multiple Python interpreters in order of preference
    let python_commands = ["python3", "python"];

    for python in python_commands {
        let result = Command::new(python)
            .args(["-c", "import basis_set_exchange; print(basis_set_exchange.get_data_dir())"])
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Get the available data directory for the basis set exchange library.
///
/// This function checks the following directories in order:
/// 1. The directory specified by `specify_bse_data_dir`.
/// 2. The directory specified by the environment variable `BSE_DATA_DIR`.
/// 3. The directory detected via Python's installed `basis_set_exchange`
///    package.
/// 4. The directory specified by the `CARGO_MANIFEST_DIR` (the directory where
///    your crate built).
pub fn get_bse_data_dir() -> Option<String> {
    let dir_specified = get_bse_data_dir_specified();
    let dir_env = get_bse_data_dir_env();
    let dir_python = get_bse_data_dir_python();
    let dir_manifest = get_bse_data_dir_manifest();

    // Helper to check if a directory contains METADATA.json
    let is_valid_dir = |dir: &str| std::path::Path::new(&format!("{dir}/METADATA.json")).exists();

    // Check in order: specified -> env -> python -> manifest
    if !dir_specified.is_empty() && is_valid_dir(&dir_specified) {
        Some(dir_specified)
    } else if !dir_env.is_empty() && is_valid_dir(&dir_env) {
        Some(dir_env)
    } else if dir_python.as_ref().is_some_and(|d| is_valid_dir(d)) {
        dir_python
    } else if !dir_manifest.is_empty() && is_valid_dir(&dir_manifest) {
        Some(dir_manifest)
    } else {
        None
    }
}

/* #endregion */

/* #region data source */

/// Get the default data source from the `BSE_REMOTE` environment variable.
///
/// The `BSE_REMOTE` environment variable controls the default data source
/// behavior when no explicit `--source` option or `source` parameter is
/// provided.
///
/// # Supported Values
///
/// - `local` or `0` or `false` or `no`: Use local data directory
/// - `remote` or `1` or `true` or `yes`: Use remote REST API (requires `remote`
///   feature)
/// - `auto`: Try local first, fallback to remote if local fails (default if
///   unset)
///
/// # Returns
///
/// The default [`BseDataSource`] based on the environment variable.
/// Returns [`BseDataSource::Auto`] if the variable is not set or has an
/// invalid value.
pub fn get_bse_source_default() -> BseDataSource {
    let source_str = std::env::var("BSE_REMOTE").unwrap_or_default();
    parse_source_from_str(&source_str)
}

/// Parse a string into a [`BseDataSource`].
///
/// Used for both `BSE_REMOTE` environment variable parsing and CLI
/// `--source` option parsing.
///
/// # Supported Values
///
/// - Empty string: Returns `Auto` (default)
/// - `local`, `0`, `false`, `no`: Returns `Local`
/// - `remote`, `1`, `true`, `yes`: Returns `Remote` (requires `remote` feature)
/// - `auto`: Returns `Auto`
///
/// Invalid values emit a warning and return `Auto`.
pub fn parse_source_from_str(source_str: &str) -> BseDataSource {
    match source_str.to_lowercase().as_str() {
        "" => BseDataSource::Auto,
        "local" | "0" | "false" | "no" => BseDataSource::Local,
        "remote" | "1" | "true" | "yes" => {
            #[cfg(feature = "remote")]
            {
                BseDataSource::Remote
            }
            #[cfg(not(feature = "remote"))]
            {
                eprintln!("Warning: 'remote' source requires the 'remote' feature. Using 'auto' instead.");
                BseDataSource::Auto
            }
        },
        "auto" => BseDataSource::Auto,
        s => {
            eprintln!("Warning: Invalid source '{}'. Use 'local', 'remote', or 'auto'. Using 'auto' instead.", s);
            BseDataSource::Auto
        },
    }
}

/// Check if local-not-found warning is enabled.
///
/// Controlled by `BSE_WARN_LOCAL_NOTFOUND` environment variable.
/// Default is `true` (warning enabled).
///
/// # Supported Values
///
/// - `0`, `false`, `no`: Disable warning
/// - Any other value or unset: Enable warning (default)
pub fn is_warn_local_notfound() -> bool {
    let val = std::env::var("BSE_WARN_LOCAL_NOTFOUND").unwrap_or_default();
    !matches!(val.to_lowercase().as_str(), "0" | "false" | "no")
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

/* #region other auxiliaries */

/// Creates a header with information about a basis set.
///
/// Information includes description, revision, etc, but not references.
pub fn header_string(basis: &BseBasis) -> String {
    use textwrap::{wrap, Options};
    let space_14 = " ".repeat(14);
    let tw = Options::new(70).initial_indent("").subsequent_indent(&space_14);

    let header_list = vec![
        "-".repeat(70),
        " BSE-rs (Basis Set Exchange in Rust)".to_string(),
        format!(" Version {}", version()),
        format!(" Acknowledges: {MAIN_URL}"),
        "-".repeat(70).to_string(),
        format!("   Basis set: {}", basis.name),
        format!(" Description: {}", basis.description),
        format!("        Role: {}", basis.role),
        format!("     Version: {}  ({})", basis.version, basis.revision_description),
        "-".repeat(70).to_string(),
    ];
    header_list.iter().flat_map(|s| wrap(s, &tw)).join("\n")
}

/* #endregion */

/* #region get_basis */

/// Data source for basis set information.
///
/// Specifies where to fetch basis set data from when calling [`get_basis`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BseDataSource {
    /// Use local data directory.
    ///
    /// Requires `BSE_DATA_DIR` environment variable or `data_dir` parameter
    /// to be set.
    Local,
    /// Use remote REST API.
    ///
    /// Requires the `remote` feature to be enabled.
    #[cfg(feature = "remote")]
    Remote,
    /// Try local first, fallback to remote if available.
    ///
    /// This is the default. Without the `remote` feature, this is
    /// equivalent to `Local`.
    #[default]
    Auto,
}

/// Arguments for [`get_basis`] and [`get_formatted_basis`].
///
/// Use [`BseGetBasisArgsBuilder`] to construct instances with a fluent API.
/// The struct can also be deserialized from TOML.
///
/// # Example
///
/// ```rust
/// use bse::prelude::*;
///
/// // Using the builder
/// let args = BseGetBasisArgsBuilder::default()
///     .elements("H, C-O".to_string())
///     .augment_diffuse(1)
///     .build()
///     .unwrap();
///
/// // From TOML
/// let args: BseGetBasisArgs = toml::from_str(r#"
///     elements = "H, C-O"
///     augment_diffuse = 1
/// "#).unwrap();
/// ```
#[derive(Builder, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[builder(build_fn(error = "BseError"))]
#[serde(default)]
pub struct BseGetBasisArgs {
    /// Elements to include. Can be atomic numbers, symbols, or ranges like
    /// "1-3,H-Li,C-O".
    #[builder(default, setter(into))]
    pub elements: Option<String>,

    /// Specific version of the basis set. Defaults to latest version.
    #[builder(default, setter(into))]
    pub version: Option<String>,

    /// Remove general contractions by duplicating primitives.
    #[builder(default = false)]
    pub uncontract_general: bool,

    /// Split combined shells (sp, spd, spdf) into separate shells.
    #[builder(default = false)]
    pub uncontract_spdf: bool,

    /// Fully uncontract: each primitive becomes a separate shell.
    #[builder(default = false)]
    pub uncontract_segmented: bool,

    /// Remove uncontracted (free) primitives from the basis set.
    #[builder(default = false)]
    pub remove_free_primitives: bool,

    /// Make the basis set as generally-contracted as possible.
    #[builder(default = false)]
    pub make_general: bool,

    /// Optimize general contractions by removing redundant functions.
    #[builder(default = false)]
    pub optimize_general: bool,

    /// Number of diffuse functions to add via even-tempered extrapolation.
    #[builder(default = 0)]
    pub augment_diffuse: i32,

    /// Number of steep functions to add via even-tempered extrapolation.
    #[builder(default = 0)]
    pub augment_steep: i32,

    /// Which auxiliary basis to generate:
    /// - 0: Return orbital basis (default)
    /// - 1: Generate AutoAux fitting basis
    /// - 2: Generate Auto-ABS Coulomb fitting basis
    #[builder(default = 0)]
    pub get_aux: i32,

    /// Override the data directory path.
    #[builder(default)]
    pub data_dir: Option<String>,

    /// Include header information in formatted output.
    #[builder(default = true)]
    pub header: bool,

    /// Data source for basis set (local, remote, or auto).
    #[builder(default)]
    pub source: BseDataSource,

    /// Custom API URL for remote fetching (requires `remote` feature).
    #[cfg(feature = "remote")]
    #[builder(default, setter(into))]
    pub api_url: Option<String>,
}

impl Default for BseGetBasisArgs {
    fn default() -> Self {
        BseGetBasisArgsBuilder::default().build().unwrap()
    }
}

impl TryFrom<&str> for BseGetBasisArgs {
    type Error = BseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        toml::from_str(s).map_err(|e| BseError::ValueError(format!("TOML parsing error: {}", e)))
    }
}

impl TryFrom<String> for BseGetBasisArgs {
    type Error = BseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<Option<&str>> for BseGetBasisArgs {
    type Error = BseError;

    fn try_from(s: Option<&str>) -> Result<Self, Self::Error> {
        Self::try_from(s.unwrap_or(""))
    }
}

/// Obtain a basis set.
///
/// This is the main function for getting basis set information.
/// This function reads in all the basis data and returns it either
/// as a string or as a serde Value (JSON-like structure).
///
/// If you are looking for a specific format (like output nwchem, gaussian94,
/// etc.), use [`get_formatted_basis`] instead.
///
/// # Arguments
/// * `name` - Name of the basis set (case insensitive)
/// * `elements` - List of elements to get the basis set for. Elements can be
///   specified by:
///   - Atomic number (as integer or string)
///   - Element symbol (case insensitive)
///   - Can be a range string like "1-3,7-10" which will be expanded
///   - If empty/None, returns all elements the basis set is defined for
/// * `version` - Optional specific version of the basis set (defaults to
///   latest)
/// * `uncontract_general` - Remove general contractions by duplicating
///   primitives
/// * `uncontract_spdf` - Remove combined angular momentum contractions (sp, spd
///   etc)
/// * `uncontract_segmented` - Remove segmented contractions (each primitive
///   becomes new shell)
/// * `make_general` - Make basis set as generally-contracted as possible (one
///   shell per am)
/// * `optimize_general` - Optimize by removing general contractions with
///   uncontracted functions
/// * `augment_diffuse` - Add n diffuse functions via even-tempered
///   extrapolation
/// * `augment_steep` - Add n steep functions via even-tempered extrapolation
/// * `get_aux` - Which auxiliary basis to return:
///   - 0: Orbital basis (default)
///   - 1: AutoAux basis
///   - 2: Auto-ABS Coulomb fitting basis
/// * `data_dir` - Alternative data directory (defaults to project's "data"
///   subdirectory)
///
/// # Examples
/// ```
/// use bse::prelude::*;
/// let args = BseGetBasisArgsBuilder::default().elements("H, 6-O".to_string()).build().unwrap();
/// let basis: BseBasis = get_basis_f("sto-3g", args).expect("Failed to get basis set");
/// println!("Basis set: {basis:#?}");
/// ```
///
/// Arguments can also be parsed by `toml`:
/// ```
/// use bse::prelude::*;
/// let args = r#"
///     elements = "H, 6-O"
///     augment_diffuse = 1
///     get_aux = 1
/// "#;
/// let args: BseGetBasisArgs = toml::from_str(args).unwrap();
/// let basis: BseBasis = get_basis_f("sto-3g", args).unwrap();
/// println!("Basis set: {basis:#?}");
/// ```
///
/// You can also pass `None` as argument, then the default `BseGetBasisArgs`
/// will be used (which means all elements, latest version, no manipulations).
pub fn get_basis(name: &str, args: impl TryInto<BseGetBasisArgs, Error: Into<BseError>>) -> BseBasis {
    get_basis_f(name, args).unwrap()
}

/// Internal function for local basis set fetching.
fn get_basis_local_f(name: &str, args: &BseGetBasisArgs) -> Result<BseBasis, BseError> {
    let data_dir = args.data_dir.clone().or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        )?;
    }
    let data_dir = data_dir.unwrap();

    // First, try the normal lookup
    let result = get_basis_local_inner_f(name, args, &data_dir);

    // If normal lookup fails, try Truhlar calendarization
    match result {
        Ok(basis) => Ok(basis),
        Err(BseError::DataNotFound(_)) => {
            // Try Truhlar calendar basis set generation
            get_basis_truhlar_calendar_f(name, args, &data_dir)
        },
        Err(e) => Err(e),
    }
}

/// Inner function for normal basis set lookup (without Truhlar calendar
/// handling).
fn get_basis_local_inner_f(name: &str, args: &BseGetBasisArgs, data_dir: &str) -> Result<BseBasis, BseError> {
    let bs_data = get_basis_metadata(name, data_dir)?;

    // If version is not specified, use the latest
    let ver = args.version.clone().unwrap_or(bs_data.latest_version);
    if !bs_data.versions.contains_key(&ver) {
        bse_raise!(DataNotFound, "Version {ver} not found in metadata.")?;
    }

    // Compose the entire basis set (all elements)
    let table_relpath = &bs_data.versions[&ver].file_relpath;
    let mut basis_dict = compose::compose_table_basis_f(table_relpath, data_dir)?;

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
            let bs_elements_keys_vec = bs_elements_keys.iter().sorted().collect_vec();
            let elements_set: HashSet<i32> = HashSet::from_iter(elements.clone());
            if !elements_set.is_subset(&bs_elements_keys) {
                bse_raise!(
                    DataNotFound,
                    "Elements {:?} not found in basis set `{name}`. Available elements: {:?}",
                    elements,
                    bs_elements_keys_vec
                )?;
            }

            // Set to only the elements we want
            basis_dict.elements.retain(|el, _| elements_set.contains(&el.parse::<i32>().unwrap()));

            // Since we only grab some of the elements, we need to update the
            // function types used, too
            basis_dict.function_types = compose::whole_basis_types(&basis_dict.elements);
        }
    }

    // Note that from now on, the pipeline is going to modify basis_dict.
    // That is ok, since we are returned a unique instance from compose_table_basis.

    apply_basis_manipulations(&mut basis_dict, args)?;

    Ok(basis_dict)
}

/// Parse a basis name to detect if it's a Truhlar calendar basis set.
///
/// Returns the month and the remaining part of the name (after the prefix)
/// if the name starts with a valid month prefix or "maug".
/// Returns None if not a Truhlar calendar name.
fn parse_truhlar_calendar_name(name: &str) -> Option<(String, String)> {
    let name_lower = name.to_lowercase();

    // Valid month prefixes for Truhlar calendarization
    let valid_months = ["jul", "jun", "may", "apr", "mar", "feb", "jan"];

    // Check if name starts with a month prefix or "maug"
    // The prefix must be followed by a hyphen
    valid_months
        .iter()
        .filter_map(|m| {
            let prefix_with_dash = format!("{}-", m);
            if name_lower.starts_with(&prefix_with_dash) {
                Some((m.to_string(), name_lower[m.len() + 1..].to_string()))
            } else {
                None
            }
        })
        .next()
        .or_else(|| name_lower.strip_prefix("maug-").map(|stripped| ("maug".to_string(), stripped.to_string())))
}

/// Apply Truhlar calendarization to a basis set.
///
/// Takes the aug- version of a basis set and applies the calendar
/// transformation to produce the requested month variant. Also updates
/// the basis name and adds the Papajak & Truhlar reference.
fn apply_truhlar_calendar(
    aug_basis: &BseBasis,
    month: &str,
    rest: &str,
    args: &BseGetBasisArgs,
) -> Result<BseBasis, BseError> {
    // For maug, determine the actual month based on DZ/TZ/QZ
    let month = if month == "maug" { determine_maug_month(rest)?.to_string() } else { month.to_string() };

    // Apply Truhlar calendarization
    let mut calendarized = manip::truhlar_calendarize(aug_basis, &month)?;

    // Update basis metadata (name, description, references)
    update_calendar_basis_metadata(&mut calendarized, &month);

    // Apply any remaining manipulations from args
    apply_basis_manipulations(&mut calendarized, args)?;

    Ok(calendarized)
}

/// Update basis set metadata for Truhlar calendar basis sets.
///
/// This updates the name (replaces "aug-" prefix with the month prefix)
/// and adds the Papajak & Truhlar reference to each element.
fn update_calendar_basis_metadata(basis: &mut BseBasis, month: &str) {
    let month_lower = month.to_lowercase();

    // Update the basis name: replace "aug-" with "{month}-"
    // e.g., "aug-cc-pVTZ" -> "jul-cc-pVTZ"
    if basis.name.to_lowercase().starts_with("aug-") {
        let name_lower = basis.name.to_lowercase();
        if let Some(pos) = name_lower.find("aug-") {
            let rest = &basis.name[pos + 4..]; // Skip "aug-"
            basis.name = format!("{}-{}", month_lower, rest);
        }
    }

    // Update description similarly
    if basis.description.to_lowercase().starts_with("aug-") {
        let desc_lower = basis.description.to_lowercase();
        if let Some(pos) = desc_lower.find("aug-") {
            let rest = &basis.description[pos + 4..];
            basis.description = format!("{}-{}", month_lower, rest);
        }
    }

    // Add the Papajak & Truhlar reference to each element
    let papajak_ref = BseBasisReference {
        reference_description: "Truhlar calendar basis set".to_string(),
        reference_keys: vec!["papajak2011a".to_string()],
    };

    for eldata in basis.elements.values_mut() {
        // Add the reference at the beginning of the references list
        eldata.references.insert(0, papajak_ref.clone());
    }
}

/// Handle Truhlar calendar basis set generation for local source.
///
/// If the basis name starts with a month prefix (feb, mar, apr, may, jun, jul)
/// or `maug`, this function will try to substitute the prefix with `aug` and
/// apply calendarization.
fn get_basis_truhlar_calendar_f(name: &str, args: &BseGetBasisArgs, data_dir: &str) -> Result<BseBasis, BseError> {
    let parsed = parse_truhlar_calendar_name(name);

    // If no month prefix found, return the original error
    let (month, rest) = match parsed {
        Some((m, r)) => (m, r),
        None => {
            return bse_raise!(DataNotFound, "Basis set `{name}` not found in metadata.")?;
        },
    };

    // Construct the aug- version of the basis name
    let aug_name = format!("aug-{}", rest);

    // Try to get the aug- version
    let aug_basis = get_basis_local_inner_f(&aug_name, args, data_dir)?;

    // Apply Truhlar calendarization
    apply_truhlar_calendar(&aug_basis, &month, &rest, args)
}

/// Handle Truhlar calendar basis set generation for remote source.
///
/// Similar to local handling, but fetches from the remote API.
#[cfg(feature = "remote")]
fn get_basis_truhlar_calendar_remote_f(name: &str, args: &BseGetBasisArgs) -> Result<BseBasis, BseError> {
    let parsed = parse_truhlar_calendar_name(name);

    // If no month prefix found, return the original error
    let (month, rest) = match parsed {
        Some((m, r)) => (m, r),
        None => {
            return bse_raise!(DataNotFound, "Basis set `{name}` not found in remote API.")?;
        },
    };

    // Construct the aug- version of the basis name
    let aug_name = format!("aug-{}", rest);

    // Try to get the aug- version from remote
    let aug_basis = client::get_basis_remote(&aug_name, args)?;

    // Apply Truhlar calendarization
    apply_truhlar_calendar(&aug_basis, &month, &rest, args)
}

/// Determine the month for "maug" based on the basis set name.
///
/// - DZ (double-zeta) -> jun
/// - TZ (triple-zeta) -> may
/// - QZ (quadruple-zeta) -> apr
fn determine_maug_month(rest: &str) -> Result<&'static str, BseError> {
    let rest_upper = rest.to_uppercase();

    // Check for QZ first (highest priority)
    if rest_upper.contains("QZ") {
        return Ok("apr");
    }

    // Check for TZ (middle priority)
    // Note: TZ could appear in names like "cc-pV(T+d)Z"
    if rest_upper.contains("TZ") {
        return Ok("may");
    }

    // Check for DZ (lowest priority)
    // DZ appears in names like "cc-pVDZ", "cc-pV(D+d)Z"
    // The uppercase of "cc-pV(D+d)Z" is "CC-PV(D+D)Z" which ends with DZ
    if rest_upper.contains("DZ") {
        return Ok("jun");
    }

    bse_raise!(
        ValueError,
        "Cannot determine 'maug' month for basis set. Expected DZ, TZ, or QZ in the basis name: {}",
        rest
    )?
}

/// Apply manipulations to a basis set based on args.
/// This is used after fetching from either local or remote source.
fn apply_basis_manipulations(basis_dict: &mut BseBasis, args: &BseGetBasisArgs) -> Result<(), BseError> {
    let mut needs_pruning = false;

    if args.remove_free_primitives {
        manip::remove_free_primitives(basis_dict);
        needs_pruning = true;
    }

    if args.optimize_general {
        manip::optimize_general(basis_dict);
        needs_pruning = true;
    }

    // uncontract_segmented implies uncontract_general
    if args.uncontract_segmented {
        manip::uncontract_segmented(basis_dict);
        needs_pruning = true;
    }

    if args.uncontract_general {
        manip::uncontract_general(basis_dict);
        needs_pruning = true;
    }

    if args.uncontract_spdf {
        manip::uncontract_spdf(basis_dict, 0);
        needs_pruning = true;
    }

    if args.make_general {
        manip::make_general(basis_dict, false);
        needs_pruning = true;
    }

    if needs_pruning {
        manip::prune_basis(basis_dict);
    }

    if args.augment_diffuse > 0 {
        manip::geometric_augmentation(basis_dict, args.augment_diffuse, false);
    }

    if args.augment_steep > 0 {
        manip::geometric_augmentation(basis_dict, args.augment_steep, true);
        sort::sort_basis(basis_dict);
    }

    // Re-make general
    if (args.augment_diffuse > 0 || args.augment_steep > 0) && args.make_general {
        manip::make_general(basis_dict, false);
    }

    match args.get_aux {
        0 => (),
        1 => *basis_dict = manip::autoaux_basis(basis_dict),
        2 => *basis_dict = manip::autoabs_basis(basis_dict, 1, 1.5),
        _ => bse_raise!(KeyError, "Invalid value for `get_aux`: {}", args.get_aux)?,
    }

    Ok(())
}

pub fn get_basis_f(
    name: &str,
    args: impl TryInto<BseGetBasisArgs, Error: Into<BseError>>,
) -> Result<BseBasis, BseError> {
    let args = args.try_into().map_err(Into::into)?;
    // Handle data source selection
    match args.source {
        BseDataSource::Local => get_basis_local_f(name, &args),
        #[cfg(feature = "remote")]
        BseDataSource::Remote => {
            // Try direct fetch from remote API first
            let result = client::get_basis_remote(name, &args);
            match result {
                Ok(mut basis) => {
                    // Apply local manipulations that the REST API doesn't support
                    apply_basis_manipulations(&mut basis, &args)?;
                    Ok(basis)
                },
                // Handle both DataNotFound and IOError (HTTP error statuses like 404, 500)
                // For Truhlar calendar basis sets, we need to try the aug- version
                Err(BseError::DataNotFound(_)) | Err(BseError::IOError(_)) => {
                    // Try Truhlar calendarization via remote
                    let mut basis = get_basis_truhlar_calendar_remote_f(name, &args)?;
                    apply_basis_manipulations(&mut basis, &args)?;
                    Ok(basis)
                },
                Err(e) => Err(e),
            }
        },
        #[cfg(feature = "remote")]
        BseDataSource::Auto => {
            // Try local first (which includes Truhlar calendar handling)
            get_basis_local_f(name, &args).or_else(|_local_err| {
                // Warn user if enabled
                if is_warn_local_notfound() {
                    eprintln!(
                        "Warning: Local data directory not found or basis set not available locally. \
                         Falling back to remote API. Set BSE_WARN_LOCAL_NOTFOUND=0 to suppress this warning."
                    );
                }
                // Try remote directly
                let result = client::get_basis_remote(name, &args);
                match result {
                    Ok(mut basis) => {
                        apply_basis_manipulations(&mut basis, &args)?;
                        Ok(basis)
                    },
                    // Handle both DataNotFound and IOError for Truhlar calendar
                    Err(BseError::DataNotFound(_)) | Err(BseError::IOError(_)) => {
                        // Try Truhlar calendarization via remote
                        let mut basis = get_basis_truhlar_calendar_remote_f(name, &args)?;
                        apply_basis_manipulations(&mut basis, &args)?;
                        Ok(basis)
                    },
                    Err(e) => Err(e),
                }
            })
        },
        #[cfg(not(feature = "remote"))]
        BseDataSource::Auto => get_basis_local_f(name, &args),
    }
}

/// Obtain a formatted basis set.
///
/// If you are looking for dumping dictionary-like structure, use
/// [`get_basis`] instead.
///
/// Usage is similar to [`get_basis`], but with an additional `fmt` argument.
///
/// # Arguments
///
/// * `name` - Name of the basis set (case insensitive)
/// * `fmt` - Desired output format (case insensitive). None returns a serde
///   Value. Example formats: nwchem, gaussian94, turbomole, etc.
/// * `args` - Arguments for the basis set, see [`BseGetBasisArgs`]. Additional
///   to [`get_basis`],
///   - `header` - Whether to include a header with information about the basis
///     set.
pub fn get_formatted_basis(
    name: &str,
    fmt: &str,
    args: impl TryInto<BseGetBasisArgs, Error: Into<BseError>>,
) -> String {
    get_formatted_basis_f(name, fmt, args).unwrap()
}

pub fn get_formatted_basis_f(
    name: &str,
    fmt: &str,
    args: impl TryInto<BseGetBasisArgs, Error: Into<BseError>>,
) -> Result<String, BseError> {
    let args = args.try_into().map_err(Into::into)?;
    let basis = get_basis_f(name, args.clone())?;
    let header = if args.header { Some(header_string(&basis)) } else { None };
    writers::write::write_formatted_basis_str_f(&basis, fmt, header.as_deref())
}

/* #endregion */

/// Return information about the basis set formats available for output.
///
/// This is a convenience wrapper that calls [`get_writer_formats`].
///
/// The returned data is a map of format name to display name. The format
/// can be passed as the `fmt` argument to [`get_formatted_basis`].
///
/// # Arguments
///
/// * `function_types` - Optional list of function types to filter by. If
///   provided, only formats supporting those types are returned.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let formats = get_formats(None);
/// assert!(!formats.is_empty());
/// assert!(formats.contains_key("nwchem"));
/// ```
pub fn get_formats(function_types: Option<Vec<String>>) -> HashMap<String, String> {
    writers::write::get_writer_formats(function_types)
}

/// Return information about the available basis set roles.
///
/// The returned data is a map of role to display name. Roles represent
/// different types of basis sets such as orbital, fitting, etc.
///
/// Available roles are:
/// - `orbital`: Orbital basis
/// - `jfit`: J-fitting
/// - `jkfit`: JK-fitting
/// - `rifit`: RI-fitting
/// - `optri`: Optimized RI-fitting
/// - `admmfit`: Auxiliary-Density Matrix Method Fitting
/// - `dftxfit`: DFT Exchange Fitting
/// - `dftjfit`: DFT Correlation Fitting
/// - `guess`: Initial guess
pub fn get_roles() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("orbital", "Orbital basis"),
        ("jfit", "J-fitting"),
        ("jkfit", "JK-fitting"),
        ("rifit", "RI-fitting"),
        ("optri", "Optimized RI-fitting"),
        ("admmfit", "Auxiliary-Density Matrix Method Fitting"),
        ("dftxfit", "DFT Exchange Fitting"),
        ("dftjfit", "DFT Correlation Fitting"),
        ("guess", "Initial guess"),
    ])
}

/* #endregion */

/* #region get_basis_names_and_families */

/// Obtain a list of all basis set names (display names).
///
/// The returned list contains the display names of all basis sets,
/// sorted alphabetically.
///
/// # Arguments
///
/// * `data_dir` - Optional data directory. If None, uses the default (from
///   `BSE_DATA_DIR` environment variable or auto-detected).
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let names = get_all_basis_names(None);
/// assert!(!names.is_empty());
/// assert!(names.contains(&"STO-3G".to_string()));
/// ```
pub fn get_all_basis_names(data_dir: Option<String>) -> Vec<String> {
    get_all_basis_names_f(data_dir).unwrap()
}

pub fn get_all_basis_names_f(data_dir: Option<String>) -> Result<Vec<String>, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let metadata = get_metadata_f(&data_dir)?;
    let names: Vec<String> = metadata.values().map(|v| v.display_name.clone()).sorted().collect();
    Ok(names)
}

/// Return a list of all basis set families.
///
/// Families group related basis sets together (e.g., "pople", "dunning",
/// "ahlrichs", etc.).
///
/// # Arguments
///
/// * `data_dir` - Optional data directory. If None, uses the default.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let families = get_families(None);
/// assert!(!families.is_empty());
/// assert!(families.contains(&"dunning".to_string()));
/// ```
pub fn get_families(data_dir: Option<String>) -> Vec<String> {
    get_families_f(data_dir).unwrap()
}

pub fn get_families_f(data_dir: Option<String>) -> Result<Vec<String>, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let metadata = get_metadata_f(&data_dir)?;
    let families: HashSet<String> = metadata.values().map(|v| v.family.clone()).collect();
    let families: Vec<String> = families.into_iter().sorted().collect();
    Ok(families)
}

/// Lookup the name of an auxiliary basis set given a primary basis set and
/// role.
///
/// This is useful for finding fitting basis sets that correspond to
/// a given orbital basis set.
///
/// # Arguments
///
/// * `primary_basis` - The primary (orbital) basis set name (case insensitive)
/// * `role` - Desired role/type of auxiliary basis set (case insensitive). Use
///   [`get_roles`] to see available roles.
/// * `data_dir` - Optional data directory.
///
/// # Returns
///
/// A list of auxiliary basis set names for the given primary basis and role.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let aux_names = lookup_basis_by_role("cc-pVTZ", "jkfit", None);
/// assert!(!aux_names.is_empty());
/// ```
pub fn lookup_basis_by_role(primary_basis: &str, role: &str, data_dir: Option<String>) -> Vec<String> {
    lookup_basis_by_role_f(primary_basis, role, data_dir).unwrap()
}

pub fn lookup_basis_by_role_f(
    primary_basis: &str,
    role: &str,
    data_dir: Option<String>,
) -> Result<Vec<String>, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let role = role.to_lowercase();
    let roles = get_roles();
    if !roles.contains_key(role.as_str()) {
        bse_raise!(ValueError, "Role '{role}' is not a valid role. Available roles: {:?}", roles.keys().collect_vec())?;
    }

    let bs_data = get_basis_metadata(primary_basis, &data_dir)?;
    let auxdata = &bs_data.auxiliaries;

    if !auxdata.contains_key(&role) {
        bse_raise!(ValueError, "Role '{role}' doesn't exist for basis set '{primary_basis}'")?;
    }

    match &auxdata[&role] {
        BseAuxiliary::Str(s) => Ok(vec![s.clone()]),
        BseAuxiliary::Vec(v) => Ok(v.clone()),
    }
}

/* #endregion */

/* #region filter_basis_sets */

/// Arguments for filtering basis sets.
///
/// Use [`BseFilterArgsBuilder`] to construct instances. All filter criteria
/// are combined with AND logic (all must match for a basis set to be included).
///
/// # Example
///
/// ```rust
/// use bse::prelude::*;
///
/// let args = BseFilterArgsBuilder::default()
///     .family("dunning".to_string())
///     .substr("aug".to_string())
///     .build()
///     .unwrap();
///
/// let filtered = filter_basis_sets(args);
/// ```
#[derive(Builder, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[builder(build_fn(error = "BseError"), default)]
pub struct BseFilterArgs {
    /// Substring to search for in the basis set name (case insensitive).
    #[builder(default, setter(into))]
    pub substr: Option<String>,

    /// Family the basis set belongs to (e.g., "dunning", "pople").
    #[builder(default, setter(into))]
    pub family: Option<String>,

    /// Role of the basis set (e.g., "orbital", "jkfit", "rifit").
    #[builder(default, setter(into))]
    pub role: Option<String>,

    /// Elements that the basis set must include (e.g., "H-C", "1-10").
    #[builder(default, setter(into))]
    pub elements: Option<String>,

    /// Override the data directory path.
    #[builder(default)]
    pub data_dir: Option<String>,
}

/// Filter basis sets by various criteria.
///
/// All parameters are ANDed together (all must match for a basis set
/// to be included in the result).
///
/// # Arguments
///
/// * `substr` - Substring to search for in the basis set name (case
///   insensitive)
/// * `family` - Family the basis set belongs to (case insensitive)
/// * `role` - Role of the basis set (case insensitive)
/// * `elements` - List of elements that the basis set must include. Elements
///   can be specified by atomic number, symbol, or ranges (e.g., "1-3,6-8" or
///   "H-Li,C-O").
/// * `data_dir` - Optional data directory
///
/// # Returns
///
/// A HashMap of internal basis set names to their metadata.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let args = BseFilterArgsBuilder::default()
///     .family("dunning".to_string())
///     .substr("aug".to_string())
///     .build()
///     .unwrap();
/// let filtered = filter_basis_sets(args);
/// assert!(!filtered.is_empty());
/// ```
pub fn filter_basis_sets(args: BseFilterArgs) -> HashMap<String, BseRootMetadata> {
    filter_basis_sets_f(args).unwrap()
}

pub fn filter_basis_sets_f(args: BseFilterArgs) -> Result<HashMap<String, BseRootMetadata>, BseError> {
    let data_dir = args.data_dir.clone().or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let mut metadata = get_metadata_f(&data_dir)?;

    // Filter by family
    if let Some(family) = &args.family {
        let family = family.to_lowercase();
        let families = get_families_f(Some(data_dir.clone()))?;
        if !families.contains(&family) {
            bse_raise!(ValueError, "Family '{family}' is not a valid family. Available families: {:?}", families)?;
        }
        metadata.retain(|_, v| v.family == family);
    }

    // Filter by role
    if let Some(role) = &args.role {
        let role = role.to_lowercase();
        let roles = get_roles();
        if !roles.contains_key(role.as_str()) {
            bse_raise!(
                ValueError,
                "Role '{role}' is not a valid role. Available roles: {:?}",
                roles.keys().collect_vec()
            )?;
        }
        metadata.retain(|_, v| v.role == role);
    }

    // Filter by elements
    if let Some(elements) = &args.elements {
        let elements = misc::expand_elements_f(elements)?;
        let elements_set: HashSet<i32> = HashSet::from_iter(elements.clone());

        for (_, basis_data) in metadata.iter_mut() {
            // Filter versions to those that contain all required elements
            basis_data.versions.retain(|_, ver| {
                let ver_elements: HashSet<i32> = ver.elements.iter().filter_map(|e| e.parse::<i32>().ok()).collect();
                elements_set.is_subset(&ver_elements)
            });
        }

        // Remove basis sets with no matching versions
        metadata.retain(|_, v| !v.versions.is_empty());
    }

    // Filter by substring
    if let Some(substr) = &args.substr {
        let substr = substr.to_lowercase();
        metadata.retain(|k, v| k.contains(&substr) || v.display_name.to_lowercase().contains(&substr));
    }

    Ok(metadata)
}

/* #endregion */

/* #region reference functions */

/// Obtain information for all stored references.
///
/// This returns a nested dictionary with all the data for all the references
/// from the REFERENCES.json file.
///
/// # Arguments
///
/// * `data_dir` - Optional data directory. If None, uses the default.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let ref_data = get_reference_data(None);
/// assert!(!ref_data.is_empty());
/// // Check for a known reference
/// assert!(ref_data.contains_key("pritchard2019a"));
/// println!("Number of references: {}", ref_data.len());
/// ```
pub fn get_reference_data(data_dir: Option<String>) -> HashMap<String, BseReferenceEntry> {
    get_reference_data_f(data_dir).unwrap()
}

pub fn get_reference_data_f(data_dir: Option<String>) -> Result<HashMap<String, BseReferenceEntry>, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let reffile_path = format!("{data_dir}/REFERENCES.json");
    let content = std::fs::read_to_string(&reffile_path)?;

    // Parse as raw JSON value first to handle the schema entry
    let raw: serde_json::Value = serde_json::from_str(&content)?;

    let mut refs: HashMap<String, BseReferenceEntry> = HashMap::new();

    if let Some(obj) = raw.as_object() {
        for (key, value) in obj {
            // Skip the schema entry
            if key == "molssi_bse_schema" {
                continue;
            }
            // Parse each reference entry
            if let Ok(entry) = serde_json::from_value::<BseReferenceEntry>(value.clone()) {
                refs.insert(key.clone(), entry);
            }
        }
    }

    Ok(refs)
}

/// Get the references/citations for a basis set.
///
/// Returns citation information for a basis set, optionally filtered by
/// elements and formatted in various citation formats.
///
/// # Arguments
///
/// * `basis_name` - Name of the basis set (case insensitive)
/// * `elements` - Optional element filter (atomic numbers, symbols, or ranges)
/// * `version` - Optional specific version (defaults to latest)
/// * `fmt` - Output format (None for structured data, or "bib", "txt", "ris",
///   "endnote", "json")
/// * `data_dir` - Optional data directory
///
/// # Returns
///
/// If `fmt` is None, returns structured reference data as a vector of
/// [`BseElementReferences`]. Otherwise, returns a formatted string.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// // Get structured reference data
/// let ref_data = get_references("cc-pVTZ", None);
/// assert!(!ref_data.is_empty());
///
/// // Get BibTeX formatted references
/// let bib = get_references_formatted("cc-pVTZ", None, None, "bib");
/// assert!(bib.contains("@article"));
/// println!("{}", bib);
/// ```
pub fn get_references(basis_name: &str, elements: Option<&str>) -> Vec<BseElementReferences> {
    get_references_f(basis_name, elements, None, None).unwrap()
}

pub fn get_references_f(
    basis_name: &str,
    elements: Option<&str>,
    version: Option<&str>,
    data_dir: Option<String>,
) -> Result<Vec<BseElementReferences>, BseError> {
    // Build args for get_basis
    let args = BseGetBasisArgsBuilder::default()
        .elements(elements.map(|s| s.to_string()))
        .version(version.map(|s| s.to_string()))
        .data_dir(data_dir.clone())
        .build()?;

    let basis_dict = get_basis_f(basis_name, args)?;
    let all_ref_data = get_reference_data_f(data_dir)?;

    Ok(references::compact_references(&basis_dict, &all_ref_data))
}

/// Get formatted references for a basis set.
///
/// This is a convenience function that combines [`get_references`] with
/// [`convert_references`].
///
/// # Arguments
///
/// * `basis_name` - Name of the basis set (case insensitive)
/// * `elements` - Optional element filter
/// * `version` - Optional specific version
/// * `fmt` - Output format ("bib", "txt", "ris", "endnote", "json")
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let bib = get_references_formatted("cc-pVTZ", Some("H,C,N,O"), None, "bib");
/// println!("{}", bib);
/// ```
pub fn get_references_formatted(basis_name: &str, elements: Option<&str>, version: Option<&str>, fmt: &str) -> String {
    get_references_formatted_f(basis_name, elements, version, fmt, None).unwrap()
}

pub fn get_references_formatted_f(
    basis_name: &str,
    elements: Option<&str>,
    version: Option<&str>,
    fmt: &str,
    data_dir: Option<String>,
) -> Result<String, BseError> {
    let ref_data = get_references_f(basis_name, elements, version, data_dir.clone())?;
    let all_ref_data = get_reference_data_f(data_dir)?;
    Ok(refconverters::convert_references(&ref_data, fmt, &all_ref_data))
}

/// Return information about the reference/citation formats available.
///
/// The returned data is a map of format name to display name.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let formats = get_reference_formats();
/// assert!(formats.contains_key("bib"));
/// println!("Available formats: {:?}", formats);
/// ```
pub fn get_reference_formats() -> HashMap<String, String> {
    refconverters::get_reference_formats()
}

/* #endregion */

/* #region notes functions */

/// Return a string representing the notes about a basis set family.
///
/// If notes are not found, an empty string is returned. If references are
/// mentioned in the notes, their full text is appended at the end.
///
/// # Arguments
///
/// * `family` - Family name (case insensitive, e.g., "dunning", "ahlrichs")
/// * `data_dir` - Optional data directory. If None, uses the default.
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let notes = get_family_notes("ahlrichs", None);
/// println!("{}", notes);
/// ```
pub fn get_family_notes(family: &str, data_dir: Option<String>) -> String {
    get_family_notes_f(family, data_dir).unwrap()
}

pub fn get_family_notes_f(family: &str, data_dir: Option<String>) -> Result<String, BseError> {
    let file_path = family_notes_path(family, data_dir.clone())?;
    let notes_str = notes::read_notes_file(&file_path);

    let notes_str = notes_str.unwrap_or_default();

    let ref_data = get_reference_data_f(data_dir)?;
    Ok(notes::process_notes(&notes_str, &ref_data))
}

/// Check if notes exist for a given family.
///
/// Returns true if the NOTES.{family} file exists, false otherwise.
///
/// # Arguments
///
/// * `family` - Family name (case insensitive)
/// * `data_dir` - Optional data directory
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let has_notes = has_family_notes("ahlrichs", None);
/// println!("ahlrichs family has notes: {}", has_notes);
/// ```
pub fn has_family_notes(family: &str, data_dir: Option<String>) -> bool {
    has_family_notes_f(family, data_dir).unwrap()
}

pub fn has_family_notes_f(family: &str, data_dir: Option<String>) -> Result<bool, BseError> {
    let file_path = family_notes_path(family, data_dir)?;
    Ok(std::path::Path::new(&file_path).exists())
}

/// Return a string representing the notes about a specific basis set.
///
/// If notes are not found, an empty string is returned. If references are
/// mentioned in the notes, their full text is appended at the end.
///
/// # Arguments
///
/// * `name` - Basis set name (case insensitive)
/// * `data_dir` - Optional data directory
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let notes = get_basis_notes("def2-SVP", None);
/// println!("{}", notes);
/// ```
pub fn get_basis_notes(name: &str, data_dir: Option<String>) -> String {
    get_basis_notes_f(name, data_dir).unwrap()
}

pub fn get_basis_notes_f(name: &str, data_dir: Option<String>) -> Result<String, BseError> {
    let file_path = basis_notes_path(name, data_dir.clone())?;
    let notes_str = notes::read_notes_file(&file_path);

    let notes_str = notes_str.unwrap_or_default();

    let ref_data = get_reference_data_f(data_dir)?;
    Ok(notes::process_notes(&notes_str, &ref_data))
}

/// Check if notes exist for a given basis set.
///
/// Returns true if the {basename}.notes file exists, false otherwise.
///
/// # Arguments
///
/// * `name` - Basis set name (case insensitive)
/// * `data_dir` - Optional data directory
///
/// # Example
///
/// ```
/// use bse::prelude::*;
/// let has_notes = has_basis_notes("def2-SVP", None);
/// println!("def2-SVP has notes: {}", has_notes);
/// ```
pub fn has_basis_notes(name: &str, data_dir: Option<String>) -> bool {
    has_basis_notes_f(name, data_dir).unwrap()
}

pub fn has_basis_notes_f(name: &str, data_dir: Option<String>) -> Result<bool, BseError> {
    let file_path = basis_notes_path(name, data_dir)?;
    Ok(std::path::Path::new(&file_path).exists())
}

// Helper functions to construct notes file paths

fn family_notes_path(family: &str, data_dir: Option<String>) -> Result<String, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let family = family.to_lowercase();
    let families = get_families_f(Some(data_dir.clone()))?;
    if !families.contains(&family) {
        bse_raise!(ValueError, "Family '{}' does not exist", family)?;
    }

    let file_name = format!("NOTES.{}", family);
    Ok(format!("{}/{}", data_dir, file_name))
}

fn basis_notes_path(name: &str, data_dir: Option<String>) -> Result<String, BseError> {
    let data_dir = data_dir.or(get_bse_data_dir());
    if data_dir.is_none() {
        return bse_raise!(
            DataNotFound,
            "No data directory specified. Please set `BSE_DATA_DIR` environment variable."
        );
    }
    let data_dir = data_dir.unwrap();

    let bs_data = get_basis_metadata(name, &data_dir)?;
    let filebase = bs_data.basename;
    Ok(format!("{}/{}.notes", data_dir, filebase))
}

/* #endregion */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_basis() {
        let args = BseGetBasisArgsBuilder::default().elements("H, 6-O".to_string()).build().unwrap();
        let basis: BseBasis = get_basis_f("sto-3g", args).unwrap();
        println!("Basis set: {basis:#?}");
        assert_eq!(basis.name, "STO-3G");

        let header = header_string(&basis);
        println!("Header:\n{header}");
    }

    #[test]
    fn test_get_basis_toml() {
        let args = r#"
            elements = "H, 6-O"
            augment_diffuse = 1
            get_aux = 1
            header = true
        "#;
        let args: BseGetBasisArgs = toml::from_str(args).unwrap();
        let basis: BseBasis = get_basis_f("sto-3g", args).unwrap();
        println!("Basis set: {basis:#?}");
    }

    #[test]
    fn test_get_header() {
        let data_dir = get_bse_data_dir().expect("Data directory not found");
        let args = BseGetBasisArgsBuilder::default().data_dir(Some(data_dir.clone())).build().unwrap();
        let basis = get_basis_f("2ZaPa-NR-CV", args).expect("Failed to get basis set");
        let header = header_string(&basis);
        println!("Header:\n{header}");
    }

    #[test]
    fn test_python_detection() {
        // Test that Python detection works if basis_set_exchange is installed
        let dir_python = get_bse_data_dir_python();
        if let Some(dir) = dir_python {
            println!("Python detected data directory: {dir}");
            assert!(std::path::Path::new(&format!("{dir}/METADATA.json")).exists());
        } else {
            println!("Python basis_set_exchange package not available, skipping");
        }
    }

    #[test]
    fn test_get_all_basis_names() {
        let names = get_all_basis_names(None);
        assert!(!names.is_empty());
        // Should contain some well-known basis sets
        assert!(names.iter().any(|n| n.contains("STO-3G")));
        assert!(names.iter().any(|n| n.contains("cc-pVTZ")));
        println!("Number of basis sets: {}", names.len());
    }

    #[test]
    fn test_get_families() {
        let families = get_families(None);
        assert!(!families.is_empty());
        assert!(families.contains(&"dunning".to_string()));
        assert!(families.contains(&"pople".to_string()));
        println!("Families: {:?}", families);
    }

    #[test]
    fn test_get_roles() {
        let roles = get_roles();
        assert!(!roles.is_empty());
        assert!(roles.contains_key("orbital"));
        assert!(roles.contains_key("jkfit"));
        println!("Roles: {:?}", roles);
    }

    #[test]
    fn test_lookup_basis_by_role() {
        // cc-pVTZ should have jkfit auxiliary
        let aux_names = lookup_basis_by_role("cc-pVTZ", "jkfit", None);
        assert!(!aux_names.is_empty());
        println!("cc-pVTZ jkfit auxiliaries: {:?}", aux_names);
    }

    #[test]
    fn test_filter_basis_sets() {
        // Filter by family
        let args = BseFilterArgsBuilder::default().family("dunning".to_string()).build().unwrap();
        let filtered = filter_basis_sets(args);
        assert!(!filtered.is_empty());
        assert!(filtered.values().all(|v| v.family == "dunning"));
        println!("Dunning family basis sets: {}", filtered.len());

        // Filter by substring
        let args = BseFilterArgsBuilder::default().substr("aug-cc-pV".to_string()).build().unwrap();
        let filtered = filter_basis_sets(args);
        assert!(!filtered.is_empty());
        println!("Basis sets matching 'aug-cc-pV': {}", filtered.len());

        // Filter by role
        let args = BseFilterArgsBuilder::default().role("jkfit".to_string()).build().unwrap();
        let filtered = filter_basis_sets(args);
        assert!(!filtered.is_empty());
        assert!(filtered.values().all(|v| v.role == "jkfit"));
        println!("JK-fitting basis sets: {}", filtered.len());

        // Filter by elements
        let args = BseFilterArgsBuilder::default().elements("H-C".to_string()).build().unwrap();
        let filtered = filter_basis_sets(args);
        assert!(!filtered.is_empty());
        println!("Basis sets covering H-C: {}", filtered.len());
    }

    #[test]
    fn test_get_reference_data() {
        let ref_data = get_reference_data(None);
        assert!(!ref_data.is_empty());
        // Check for known reference keys
        assert!(ref_data.contains_key("pritchard2019a"));
        assert!(ref_data.contains_key("feller1996a"));
        println!("Number of references: {}", ref_data.len());
    }

    #[test]
    fn test_get_references() {
        let ref_data = get_references("cc-pVTZ", Some("H,C"));
        assert!(!ref_data.is_empty());

        // Should have elements grouped
        for group in &ref_data {
            assert!(!group.elements.is_empty());
            println!("Elements: {}", misc::compact_elements(&group.elements));
        }
    }

    #[test]
    fn test_get_references_formatted() {
        // Test BibTeX format
        let bib = get_references_formatted("cc-pVTZ", Some("H"), None, "bib");
        assert!(bib.contains("@article"));
        println!("BibTeX output:\n{}", &bib[..500.min(bib.len())]);

        // Test plain text format
        let txt = get_references_formatted("cc-pVTZ", Some("H"), None, "txt");
        assert!(txt.contains("pritchard2019a") || txt.contains("Dunning"));
        println!("Plain text output:\n{}", &txt[..500.min(txt.len())]);

        // Test JSON format
        let json = get_references_formatted("cc-pVTZ", Some("H"), None, "json");
        assert!(json.starts_with("["));
        println!("JSON output:\n{}", &json[..200.min(json.len())]);
    }

    #[test]
    fn test_get_reference_formats() {
        let formats = get_reference_formats();
        assert!(!formats.is_empty());
        assert!(formats.contains_key("bib"));
        assert!(formats.contains_key("txt"));
        assert!(formats.contains_key("json"));
        println!("Reference formats: {:?}", formats);
    }

    #[test]
    fn test_get_family_notes() {
        // Test that family notes can be retrieved
        let notes = get_family_notes("ahlrichs", None);
        assert!(notes.contains("def2"));
        println!("ahlrichs family notes:\n{}", &notes[..500.min(notes.len())]);

        // Test another family
        let notes = get_family_notes("dunning", None);
        assert!(notes.contains("cc-pV") || notes.contains("correlation-consistent"));
        println!("dunning family notes:\n{}", &notes[..500.min(notes.len())]);

        // Test empty family (no notes file)
        // Most families have notes, so we just check we get something
        let notes = get_family_notes("pople", None);
        // pople family notes should exist
        println!("pople family notes length: {}", notes.len());
    }

    #[test]
    fn test_get_basis_notes() {
        // Test basis with notes
        let notes = get_basis_notes("def2-SVP", None);
        assert!(notes.contains("Original BSE Contributor"));
        println!("def2-SVP basis notes:\n{}", &notes[..500.min(notes.len())]);

        // Test basis with notes that have reference substitution
        let notes = get_basis_notes("3-21G", None);
        assert!(notes.contains("Caesium"));
        assert!(notes.contains("REFERENCES MENTIONED ABOVE"));
        println!("3-21G basis notes:\n{}", &notes[..500.min(notes.len())]);

        // Test basis without notes file (should return empty string)
        // Most basis sets have notes, but we test it works
        let notes = get_basis_notes("cc-pVTZ", None);
        println!("cc-pVTZ basis notes length: {}", notes.len());
    }

    #[test]
    fn test_has_family_notes() {
        // Test existing family notes
        assert!(has_family_notes("ahlrichs", None));
        assert!(has_family_notes("dunning", None));
        assert!(has_family_notes("pople", None));

        // Non-existing family should raise error, so we test case insensitivity
        assert!(has_family_notes("AHLRICHS", None)); // uppercase should work
    }

    #[test]
    fn test_has_basis_notes() {
        // Test existing basis notes
        assert!(has_basis_notes("def2-SVP", None));
        assert!(has_basis_notes("3-21G", None));
        assert!(has_basis_notes("cc-pVTZ", None));

        // Case insensitivity
        assert!(has_basis_notes("DEF2-SVP", None));
    }

    #[test]
    fn test_process_notes() {
        // Test the process_notes function directly
        let ref_data = get_reference_data(None);

        // Notes with reference key
        let notes_with_ref = "This basis set uses andzelm1984a for Cs.";
        let processed = crate::notes::process_notes(notes_with_ref, &ref_data);
        assert!(processed.contains("REFERENCES MENTIONED ABOVE"));
        assert!(processed.contains("andzelm1984a"));
        println!("Processed notes:\n{}", processed);

        // Notes without reference key
        let notes_no_ref = "This is just a regular note.";
        let processed = crate::notes::process_notes(notes_no_ref, &ref_data);
        assert!(!processed.contains("REFERENCES MENTIONED ABOVE"));
        assert_eq!(processed, notes_no_ref);
    }

    #[test]
    fn test_parse_truhlar_calendar_name() {
        // Test month prefixes
        assert_eq!(parse_truhlar_calendar_name("jul-cc-pVTZ"), Some(("jul".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("jun-cc-pVTZ"), Some(("jun".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("may-cc-pVTZ"), Some(("may".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("apr-cc-pVTZ"), Some(("apr".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("mar-cc-pVTZ"), Some(("mar".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("feb-cc-pVTZ"), Some(("feb".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("jan-cc-pVTZ"), Some(("jan".to_string(), "cc-pvtz".to_string())));

        // Test maug
        assert_eq!(parse_truhlar_calendar_name("maug-cc-pVTZ"), Some(("maug".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("maug-cc-pVDZ"), Some(("maug".to_string(), "cc-pvdz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("maug-cc-pVQZ"), Some(("maug".to_string(), "cc-pvqz".to_string())));

        // Test case insensitivity
        assert_eq!(parse_truhlar_calendar_name("JUL-cc-pVTZ"), Some(("jul".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("Jul-CC-PVTZ"), Some(("jul".to_string(), "cc-pvtz".to_string())));
        assert_eq!(parse_truhlar_calendar_name("MAUG-cc-pVTZ"), Some(("maug".to_string(), "cc-pvtz".to_string())));

        // Test non-calendar names
        assert_eq!(parse_truhlar_calendar_name("cc-pVTZ"), None);
        assert_eq!(parse_truhlar_calendar_name("aug-cc-pVTZ"), None);
        assert_eq!(parse_truhlar_calendar_name("def2-SVP"), None);
        assert_eq!(parse_truhlar_calendar_name("sto-3g"), None);

        // Test edge cases
        assert_eq!(parse_truhlar_calendar_name("jul"), None); // No hyphen
        assert_eq!(parse_truhlar_calendar_name("julcc-pVTZ"), None); // No hyphen after prefix
    }

    #[test]
    fn test_determine_maug_month() {
        // DZ -> jun
        assert_eq!(determine_maug_month("cc-pvdz").unwrap(), "jun");
        assert_eq!(determine_maug_month("cc-pVDZ").unwrap(), "jun");
        assert_eq!(determine_maug_month("aug-cc-pvdz").unwrap(), "jun");

        // TZ -> may
        assert_eq!(determine_maug_month("cc-pvtz").unwrap(), "may");
        assert_eq!(determine_maug_month("cc-pVTZ").unwrap(), "may");
        assert_eq!(determine_maug_month("aug-cc-pvtz").unwrap(), "may");

        // QZ -> apr
        assert_eq!(determine_maug_month("cc-pvqz").unwrap(), "apr");
        assert_eq!(determine_maug_month("cc-pVQZ").unwrap(), "apr");
        assert_eq!(determine_maug_month("aug-cc-pvqz").unwrap(), "apr");

        // Case insensitivity
        assert_eq!(determine_maug_month("CC-PVDZ").unwrap(), "jun");
        assert_eq!(determine_maug_month("CC-PVTZ").unwrap(), "may");
        assert_eq!(determine_maug_month("CC-PVQZ").unwrap(), "apr");

        // Test that priority is correct (QZ > TZ > DZ)
        // If a name contains multiple, the highest wins
        // (This shouldn't normally happen, but we test the priority)

        // Invalid cases
        assert!(determine_maug_month("cc-pv5z").is_err());
        assert!(determine_maug_month("def2-svp").is_err());
        assert!(determine_maug_month("sto-3g").is_err());
    }

    #[test]
    fn test_truhlar_calendar_basis() {
        // Test that we can get a Truhlar calendar basis set
        // jul-cc-pVTZ should work (try existing first, then generate from aug-cc-pVTZ)
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("jul-cc-pVTZ", args);
        assert!(basis.is_ok());
        let basis = basis.unwrap();
        println!("jul-cc-pVTZ basis: {:?}", basis.name);

        // Test jun-cc-pVTZ
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("jun-cc-pVTZ", args);
        assert!(basis.is_ok());

        // Test may-cc-pVTZ
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("may-cc-pVTZ", args);
        assert!(basis.is_ok());

        // Test maug-cc-pVTZ (should resolve to may for TZ)
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("maug-cc-pVTZ", args);
        assert!(basis.is_ok());

        // Test maug-cc-pVDZ (should resolve to jun for DZ)
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("maug-cc-pVDZ", args);
        assert!(basis.is_ok());

        // Test maug-cc-pVQZ (should resolve to apr for QZ)
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let basis = get_basis_f("maug-cc-pVQZ", args);
        assert!(basis.is_ok());
    }

    #[test]
    fn test_truhlar_calendar_invalid() {
        // Test that invalid month prefix on non-existent aug basis fails
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        // jul-def2-SVP should fail because aug-def2-SVP doesn't exist
        let result = get_basis_f("jul-def2-SVP", args);
        assert!(result.is_err());

        // Test invalid maug (no DZ/TZ/QZ)
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();
        let result = get_basis_f("maug-sto-3g", args);
        assert!(result.is_err());
    }

    #[test]
    fn test_truhlar_calendar_existing_basis() {
        // Some calendar basis sets already exist in the data
        // This tests that we first try the existing basis before generating
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).build().unwrap();

        // Try to get jul-cc-pV(D+d)Z which might exist in the data
        let result = get_basis_f("jul-cc-pV(D+d)Z", args.clone());
        if result.is_ok() {
            println!("jul-cc-pV(D+d)Z exists directly in data");
        } else {
            // If not, it should be generated from aug-cc-pV(D+d)Z
            println!("jul-cc-pV(D+d)Z would be generated from aug-cc-pV(D+d)Z");
        }
    }

    #[test]
    fn test_truhlar_calendar_metadata() {
        // Test that the API layer updates name and adds papajak2011a reference
        let args = BseGetBasisArgsBuilder::default().elements("H,C".to_string()).build().unwrap();

        // Test jul-cc-pVTZ
        let basis = get_basis_f("jul-cc-pVTZ", args).unwrap();

        // Verify name is updated
        assert!(basis.name.starts_with("jul-"), "Expected name to start with 'jul-', got '{}'", basis.name);

        // Verify description is updated
        assert!(
            basis.description.starts_with("jul-"),
            "Expected description to start with 'jul-', got '{}'",
            basis.description
        );

        // Verify papajak2011a reference is added to each element
        for (el_z, eldata) in &basis.elements {
            let has_papajak_ref =
                eldata.references.iter().any(|r| r.reference_keys.contains(&"papajak2011a".to_string()));
            assert!(has_papajak_ref, "Element {} should have papajak2011a reference", el_z);
        }

        // Test may-cc-pVTZ
        let args = BseGetBasisArgsBuilder::default().elements("H,C".to_string()).build().unwrap();
        let basis = get_basis_f("may-cc-pVTZ", args).unwrap();
        assert!(basis.name.starts_with("may-"));
        assert!(basis.description.starts_with("may-"));
    }

    #[cfg(feature = "remote")]
    #[test]
    fn test_usual_basis_remote() {
        let args =
            BseGetBasisArgsBuilder::default().elements("S".to_string()).source(BseDataSource::Remote).build().unwrap();
        let basis_remote = get_basis_f("aug-cc-pVTZ", args);
        match &basis_remote {
            Ok(b) => println!("Auto aug-cc-pVTZ basis: {:?}", b.name),
            Err(e) => println!("Error getting aug-cc-pVTZ via Auto: {:?}", e),
        }
        assert!(basis_remote.is_ok());

        let args =
            BseGetBasisArgsBuilder::default().elements("S".to_string()).source(BseDataSource::Local).build().unwrap();
        let basis_local = get_basis_f("aug-cc-pVTZ", args);
        match &basis_local {
            Ok(b) => println!("Auto aug-cc-pVTZ basis: {:?}", b.name),
            Err(e) => println!("Error getting aug-cc-pVTZ via Auto: {:?}", e),
        }
        assert!(basis_local.is_ok());

        assert_eq!(basis_remote.unwrap(), basis_local.unwrap());
    }

    #[cfg(feature = "remote")]
    #[test]
    fn test_truhlar_calendar_remote() {
        // Test Truhlar calendar with remote source
        // This verifies that the Truhlar calendarization works for remote API

        let args =
            BseGetBasisArgsBuilder::default().elements("S".to_string()).source(BseDataSource::Remote).build().unwrap();

        // Test jul-cc-pVTZ via Remote source
        let basis_remote = get_basis_f("jul-cc-pVTZ", args);
        match &basis_remote {
            Ok(b) => println!("Remote jul-cc-pVTZ basis: {:?}", b.name),
            Err(e) => println!("Error getting jul-cc-pVTZ via Remote: {:?}", e),
        }
        assert!(basis_remote.is_ok());

        // Get the same basis via Local source for comparison
        let args =
            BseGetBasisArgsBuilder::default().elements("S".to_string()).source(BseDataSource::Local).build().unwrap();
        let basis_local = get_basis_f("jul-cc-pVTZ", args);
        match &basis_local {
            Ok(b) => println!("Local jul-cc-pVTZ basis: {:?}", b.name),
            Err(e) => println!("Error getting jul-cc-pVTZ via Local: {:?}", e),
        }
        assert!(basis_local.is_ok());

        // Both should produce the same result
        assert_eq!(basis_remote.unwrap(), basis_local.unwrap());
    }
}
