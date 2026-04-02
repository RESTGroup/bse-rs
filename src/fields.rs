//! Data structures for basis set information.
//!
//! This module defines the core data structures used to represent basis sets,
//! their components, and associated metadata. The structures are designed to
//! be compatible with the JSON format used by the MolSSI Basis Set Exchange.
//!
//! # Main Types
//!
//! - [`BseBasis`] - Complete basis set with all elements and metadata
//! - [`BseBasisElement`] - Basis data for a single element
//! - [`BseElectronShell`] - A single electron shell (e.g., 1s, 2p)
//! - [`BseEcpPotential`] - An ECP potential component
//! - [`BseRootMetadata`] - Metadata for a basis set from METADATA.json

use crate::prelude::*;

/* #region field for components */

/// Auxiliary basis set reference.
///
/// Represents an auxiliary basis set associated with a primary basis set.
/// Can be either a single basis set name or a list of names.
#[derive(Debug, Clone, PartialEq)]
pub enum BseAuxiliary {
    /// Single auxiliary basis set name.
    Str(String),
    /// Multiple auxiliary basis set names.
    Vec(Vec<String>),
}

/// Electron shell data for a Gaussian-type orbital.
///
/// Represents a single shell in a basis set, containing the angular momentum,
/// exponents, and contraction coefficients.
///
/// # Example
///
/// A typical s-shell might have:
/// - `angular_momentum: [0]` (s orbital)
/// - `exponents: ["1.234e+01", "4.567e+00"]` (two Gaussian primitives)
/// - `coefficients: [["0.123", "0.456"]]` (one contraction)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseElectronShell {
    /// Function type (e.g., "gto", "gto_spherical", "gto_cartesian").
    pub function_type: String,
    /// Region identifier for the shell.
    pub region: String,
    /// Angular momentum values. For combined shells (sp, spd), contains
    /// multiple values.
    pub angular_momentum: Vec<i32>,
    /// Gaussian exponents as strings.
    pub exponents: Vec<String>,
    /// Contraction coefficients as strings. Each inner vector is one
    /// contraction.
    pub coefficients: Vec<Vec<String>>,
}

/// GTO basis data for a single element from a component file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseGtoElement {
    /// Reference keys for this element's data.
    pub references: Vec<String>,
    /// Electron shells for this element.
    pub electron_shells: Vec<BseElectronShell>,
}

/// ECP potential component for a single angular momentum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseEcpPotential {
    /// Angular momentum for this potential.
    pub angular_momentum: Vec<i32>,
    /// Contraction coefficients.
    pub coefficients: Vec<Vec<String>>,
    /// ECP type (e.g., "scalar_ecp").
    pub ecp_type: String,
    /// R exponents for the ECP.
    pub r_exponents: Vec<i32>,
    /// Gaussian exponents for the ECP.
    pub gaussian_exponents: Vec<String>,
}

/// ECP basis data for a single element from a component file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseEcpElement {
    /// Reference keys for this element's data.
    pub references: Vec<String>,
    /// Number of electrons replaced by the ECP.
    pub ecp_electrons: i32,
    /// ECP potentials for this element.
    pub ecp_potentials: Vec<BseEcpPotential>,
}

/* #endregion */

/* #region field for skeletons */

/// Schema information for MolSSI BSE JSON files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BseMolssiBseSchema {
    /// Schema type (e.g., "component", "element", "table", "complete").
    pub schema_type: String,
    /// Schema version string.
    pub schema_version: String,
}

/// GTO component file data for multiple elements.
///
/// Represents the contents of a component JSON file containing GTO basis
/// data for multiple elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentGto {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Description of this component.
    pub description: String,
    /// Data source identifier.
    pub data_source: String,
    /// Element data indexed by atomic number string.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseGtoElement>,
}

/// ECP component file data for multiple elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentEcp {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Description of this component.
    pub description: String,
    /// Data source identifier.
    pub data_source: String,
    /// Element data indexed by atomic number string.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseEcpElement>,
}

/// Component references for a single element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseElementComponents {
    /// List of component file paths.
    pub components: Vec<String>,
}

/// Element file data mapping elements to their components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelElement {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Name of this element basis.
    pub name: String,
    /// Description.
    pub description: String,
    /// Element to component mapping.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseElementComponents>,
}

/// Table file data mapping elements to their element files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelTable {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Revision description.
    pub revision_description: String,
    /// Revision date.
    pub revision_date: String,
    /// Element to element file mapping.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, String>,
}

/// Metadata file data for a basis set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelMetadata {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Alternative names for this basis set.
    pub names: Vec<String>,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Family name (e.g., "dunning", "pople").
    pub family: String,
    /// Human-readable description.
    pub description: String,
    /// Role (e.g., "orbital", "jkfit", "rifit").
    pub role: String,
    /// Associated auxiliary basis sets.
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
}

/* #endregion */

/* #region METADATA.json */

/// Version-specific metadata for a basis set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseRootMetadataVer {
    /// Relative path to the table file.
    pub file_relpath: String,
    /// Revision description.
    pub revdesc: String,
    /// Revision date.
    pub revdate: String,
    /// List of element atomic numbers this version supports.
    pub elements: Vec<String>,
}

/// Root metadata for a basis set from METADATA.json.
///
/// Contains all information about a basis set needed for lookup and
/// retrieval, including versions, elements, and auxiliary basis sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseRootMetadata {
    /// Human-readable display name.
    pub display_name: String,
    /// Alternative names for searching.
    pub other_names: Vec<String>,
    /// Human-readable description.
    pub description: String,
    /// Latest version identifier.
    pub latest_version: String,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Base filename.
    pub basename: String,
    /// Relative path to the basis set files.
    pub relpath: String,
    /// Family name (e.g., "dunning", "pople").
    pub family: String,
    /// Role (e.g., "orbital", "jkfit", "rifit").
    pub role: String,
    /// Function types supported (e.g., "gto", "gto_spherical").
    pub function_types: Vec<String>,
    /// Associated auxiliary basis sets by role.
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    /// Available versions indexed by version string.
    #[serde(serialize_with = "ordered_map")]
    pub versions: HashMap<String, BseRootMetadataVer>,
}

/* #endregion */

/* #region Bse high-level fields */

/// Reference information for basis set data sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseBasisReference {
    /// Description of the data source.
    pub reference_description: String,
    /// Reference keys for citation lookup.
    pub reference_keys: Vec<String>,
}

/// Basis set data for a single element.
///
/// Contains the electron shells and/or ECP potentials for one element,
/// along with reference information.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BseBasisElement {
    /// Reference information for this element's data.
    pub references: Vec<BseBasisReference>,
    /// Electron shells for this element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub electron_shells: Option<Vec<BseElectronShell>>,
    /// ECP potentials for this element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecp_potentials: Option<Vec<BseEcpPotential>>,
    /// Number of electrons replaced by ECP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecp_electrons: Option<i32>,
}

/// Complete basis set information.
///
/// The main data structure returned by [`get_basis`],
/// containing all elements, shells, and metadata for a basis set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BseBasis {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Revision description.
    pub revision_description: String,
    /// Revision date.
    pub revision_date: String,
    /// Element data indexed by atomic number string.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseBasisElement>,
    /// Version string.
    pub version: String,
    /// Function types in this basis set.
    pub function_types: Vec<String>,
    /// Alternative names.
    pub names: Vec<String>,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Family name.
    pub family: String,
    /// Human-readable description.
    pub description: String,
    /// Role (orbital, jkfit, etc.).
    pub role: String,
    /// Associated auxiliary basis sets.
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    /// Display name.
    pub name: String,
}

/// Minimal basis set information for reader output.
///
/// Returned by
/// [`read_formatted_basis_str`],
/// containing parsed basis set data without full metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BseBasisMinimal {
    /// Schema information.
    pub molssi_bse_schema: BseMolssiBseSchema,
    /// Element data indexed by atomic number string.
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseBasisElement>,
    /// Function types in this basis set.
    pub function_types: Vec<String>,
    /// Basis set name.
    pub name: String,
    /// Description.
    pub description: String,
}

impl BseBasis {
    /// Create a BseBasis from a BseBasisMinimal.
    ///
    /// This is useful for readers that return minimal basis set data,
    /// which can then be used with manipulation functions that require
    /// full BseBasis.
    pub fn from_minimal(minimal: BseBasisMinimal) -> Self {
        BseBasis {
            molssi_bse_schema: minimal.molssi_bse_schema,
            revision_description: String::new(),
            revision_date: String::new(),
            elements: minimal.elements,
            version: String::new(),
            function_types: minimal.function_types,
            names: vec![minimal.name.clone()],
            tags: Vec::new(),
            family: String::new(),
            description: minimal.description,
            role: "orbital".to_string(),
            auxiliaries: HashMap::new(),
            name: minimal.name,
        }
    }
}

/* #endregion */

/* #region ser/de implementation */

/// Serialize a HashMap with keys sorted alphabetically.
///
/// Used with serde's `#[serde(serialize_with)]` attribute to produce
/// consistent JSON output.
fn ordered_map<S, K: Ord + Serialize, V: Serialize>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<&K, &V> = value.iter().collect();
    ordered.serialize(serializer)
}

/// Serialize a HashMap with string keys sorted as integers.
///
/// Used for element maps where keys are atomic number strings ("1", "2", etc.).
fn ordered_i32_map<S, V: Serialize>(value: &HashMap<String, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<IntString, &V> = value.iter().map(|(k, v)| (IntString(k.clone()), v)).collect();
    ordered.serialize(serializer)
}

/// Wrapper for sorting strings as integers.
#[derive(Debug)]
struct IntString(String);

impl PartialEq for IntString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IntString {}

impl PartialOrd for IntString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_num: i32 = self.0.parse().expect("Invalid integer string");
        let other_num: i32 = other.0.parse().expect("Invalid integer string");
        self_num.cmp(&other_num)
    }
}

impl Serialize for IntString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for IntString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(IntString(s))
    }
}

impl<'de> Deserialize<'de> for BseAuxiliary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value: Value = Value::deserialize(deserializer)?;
        match value {
            Value::String(v) => Ok(BseAuxiliary::Str(v)),
            Value::Array(arr) => Ok(BseAuxiliary::Vec(arr.iter().map(|v| v.to_string()).collect())),
            _ => Err(D::Error::custom("Expected a string or an array of strings")),
        }
    }
}

impl Serialize for BseAuxiliary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BseAuxiliary::Str(s) => s.serialize(serializer),
            BseAuxiliary::Vec(v) => v.serialize(serializer),
        }
    }
}

pub fn deserialize_auxiliary_map<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let auxiliaries: HashMap<String, BseAuxiliary> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for (key, value) in auxiliaries {
        match value {
            BseAuxiliary::Str(s) => result.insert(key, vec![s]),
            BseAuxiliary::Vec(v) => result.insert(key, v),
        };
    }
    Ok(result)
}

/* #endregion */

/* #region cached read json */

/// Read a GTO component file (panicking version).
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn read_skel_component_gto_file(element_relpath: &str, data_dir: &str) -> BseSkelComponentGto {
    read_skel_component_gto_file_f(element_relpath, data_dir).unwrap()
}

/// Read an ECP component file (panicking version).
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn read_skel_component_ecp_file(element_relpath: &str, data_dir: &str) -> BseSkelComponentEcp {
    read_skel_component_ecp_file_f(element_relpath, data_dir).unwrap()
}

/// Read an element file (panicking version).
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn read_skel_element_file(skel_element_relpath: &str, data_dir: &str) -> BseSkelElement {
    read_skel_element_file_f(skel_element_relpath, data_dir).unwrap()
}

/// Read a table file (panicking version).
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn read_skel_table_file(skel_table_relpath: &str, data_dir: &str) -> BseSkelTable {
    read_skel_table_file_f(skel_table_relpath, data_dir).unwrap()
}

/// Read a metadata file (panicking version).
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn read_skel_metadata_file(skel_metadata_relpath: &str, data_dir: &str) -> BseSkelMetadata {
    read_skel_metadata_file_f(skel_metadata_relpath, data_dir).unwrap()
}

/// Read and parse a GTO component JSON file.
///
/// Results are cached for performance.
///
/// # Arguments
///
/// * `element_relpath` - Relative path to the component file
/// * `data_dir` - Base data directory path
#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{element_relpath}")} "#)]
pub fn read_skel_component_gto_file_f(element_relpath: &str, data_dir: &str) -> Result<BseSkelComponentGto, BseError> {
    let path = format!("{data_dir}/{element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

/// Read and parse an ECP component JSON file.
///
/// Results are cached for performance.
#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{element_relpath}")} "#)]
pub fn read_skel_component_ecp_file_f(element_relpath: &str, data_dir: &str) -> Result<BseSkelComponentEcp, BseError> {
    let path = format!("{data_dir}/{element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

/// Read and parse an element JSON file.
///
/// Results are cached for performance.
#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_element_relpath}")} "#)]
pub fn read_skel_element_file_f(skel_element_relpath: &str, data_dir: &str) -> Result<BseSkelElement, BseError> {
    let path = format!("{data_dir}/{skel_element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

/// Read and parse a table JSON file.
///
/// Results are cached for performance.
#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_table_relpath}")} "#)]
pub fn read_skel_table_file_f(skel_table_relpath: &str, data_dir: &str) -> Result<BseSkelTable, BseError> {
    let path = format!("{data_dir}/{skel_table_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

/// Read and parse a metadata JSON file.
///
/// Results are cached for performance.
#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_metadata_relpath}")} "#)]
pub fn read_skel_metadata_file_f(skel_metadata_relpath: &str, data_dir: &str) -> Result<BseSkelMetadata, BseError> {
    let path = format!("{data_dir}/{skel_metadata_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

/* #endregion */

/* #region Reference data structures */

/// A single reference entry from REFERENCES.json.
///
/// Each reference contains an entry type (article, book, thesis, etc.)
/// and a set of fields (authors, title, journal, year, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseReferenceEntry {
    /// BibTeX entry type (article, book, phdthesis, etc.).
    #[serde(rename = "_entry_type")]
    pub entry_type: String,
    /// Reference fields as a HashMap.
    /// Field values can be either a single string or an array of strings.
    #[serde(flatten)]
    pub fields: HashMap<String, BseReferenceField>,
}

impl BseReferenceEntry {
    /// Get a field value as a vector of strings.
    ///
    /// For single-string fields, returns a single-element vector.
    pub fn get_field(&self, key: &str) -> Vec<String> {
        self.fields.get(key).map(|f| f.to_vec()).unwrap_or_default()
    }

    /// Get a field value as a single string, if present.
    ///
    /// For array fields, returns the first element.
    pub fn get_field_opt(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(|f| f.first())
    }
}

/// Reference field value.
///
/// Can be either a single string or an array of strings.
/// This is needed because reference fields are heterogeneous in the JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum BseReferenceField {
    /// Single string value.
    Str(String),
    /// Array of string values (e.g., authors list).
    Vec(Vec<String>),
}

impl BseReferenceField {
    /// Convert to a vector of strings.
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            BseReferenceField::Str(s) => vec![s.clone()],
            BseReferenceField::Vec(v) => v.clone(),
        }
    }

    /// Get the first string value.
    pub fn first(&self) -> Option<String> {
        match self {
            BseReferenceField::Str(s) => Some(s.clone()),
            BseReferenceField::Vec(v) => v.first().cloned(),
        }
    }
}

impl<'de> Deserialize<'de> for BseReferenceField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value: Value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) => Ok(BseReferenceField::Str(s)),
            Value::Array(arr) => {
                Ok(BseReferenceField::Vec(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()))
            },
            _ => Err(D::Error::custom("Expected a string or an array of strings")),
        }
    }
}

impl Serialize for BseReferenceField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BseReferenceField::Str(s) => s.serialize(serializer),
            BseReferenceField::Vec(v) => v.serialize(serializer),
        }
    }
}

/// Compacted element references group.
///
/// Groups elements that share the same reference information together,
/// with the actual reference data attached.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseElementReferences {
    /// Atomic numbers of elements in this group.
    pub elements: Vec<i32>,
    /// Reference information for this element group.
    pub reference_info: Vec<BseReferenceInfoWithData>,
}

/// Reference info with actual reference data attached.
///
/// Combines the reference description with the full reference data
/// for each reference key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseReferenceInfoWithData {
    /// Human-readable description of this reference.
    pub reference_description: String,
    /// Reference data (key -> entry mapping).
    pub reference_data: Vec<(String, BseReferenceEntry)>,
}

/* #endregion */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_jsons() {
        let bse_data_dir = get_bse_data_dir().unwrap();

        // skeleton components, gto
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/ahlrichs/TZV/def2-TZVP-base.1.json")).unwrap();
        let data: BseSkelComponentGto = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton components, ecp
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/ahlrichs/ECP/def2-ECP.1.json")).unwrap();
        let data: BseSkelComponentEcp = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton elements
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/ahlrichs/def2-QZVPP.1.element.json")).unwrap();
        let data: BseSkelElement = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton table
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/def2-QZVPP.1.table.json")).unwrap();
        let data: BseSkelTable = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton metadata
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/def2-QZVPP.metadata.json")).unwrap();
        let data: BseSkelMetadata = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // root metadata
        let json_data = std::fs::read_to_string(format!("{bse_data_dir}/METADATA.json")).unwrap();
        let _data: HashMap<String, BseRootMetadata> = serde_json::from_str(&json_data).unwrap();
        // println!("{data:?}");
    }
}
