//! Field definitions.

use crate::prelude::*;

/* #region field for components */

#[derive(Debug, Clone, PartialEq)]
pub enum BseAuxiliary {
    Str(String),
    Vec(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseElectronShell {
    pub function_type: String,
    pub region: String,
    pub angular_momentum: Vec<i32>,
    pub exponents: Vec<String>,
    pub coefficients: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseGtoElement {
    pub references: Vec<String>,
    pub electron_shells: Vec<BseElectronShell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseEcpPotential {
    pub angular_momentum: Vec<i32>,
    pub coefficients: Vec<Vec<String>>,
    pub ecp_type: String,
    pub r_exponents: Vec<i32>,
    pub gaussian_exponents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseEcpElement {
    pub references: Vec<String>,
    pub ecp_electrons: i32,
    pub ecp_potentials: Vec<BseEcpPotential>,
}

/* #endregion */

/* #region field for skeletons */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseMolssiBseSchema {
    pub schema_type: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentGto {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseGtoElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentEcp {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseEcpElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseElementComponents {
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelElement {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub name: String,
    pub description: String,
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseElementComponents>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelTable {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub revision_description: String,
    pub revision_date: String,
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelMetadata {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub family: String,
    pub description: String,
    pub role: String,
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
}

/* #endregion */

/* #region METADATA.json */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseRootMetadataVer {
    pub file_relpath: String,
    pub revdesc: String,
    pub revdate: String,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseRootMetadata {
    pub display_name: String,
    pub other_names: Vec<String>,
    pub description: String,
    pub latest_version: String,
    pub tags: Vec<String>,
    pub basename: String,
    pub relpath: String,
    pub family: String,
    pub role: String,
    pub function_types: Vec<String>,
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    #[serde(serialize_with = "ordered_map")]
    pub versions: HashMap<String, BseRootMetadataVer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseBasisReference {
    pub reference_description: String,
    pub reference_keys: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BseBasisElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub electron_shells: Option<Vec<BseElectronShell>>,
    pub references: Vec<BseBasisReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecp_potentials: Option<Vec<BseEcpPotential>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecp_electrons: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseBasis {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub revision_description: String,
    pub revision_date: String,
    #[serde(serialize_with = "ordered_i32_map")]
    pub elements: HashMap<String, BseBasisElement>,
    pub version: String,
    pub function_types: Vec<String>,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub family: String,
    pub description: String,
    pub role: String,
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    pub name: String,
}

/* #endregion */

/* #region ser/de implementation */

/// For use with serde's [serialize_with] attribute
/// <https://stackoverflow.com/questions/42723065/how-to-sort-hashmap-keys-when-serializing-with-serde>
fn ordered_map<S, K: Ord + Serialize, V: Serialize>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<&K, &V> = value.iter().collect();
    ordered.serialize(serializer)
}

/// For use with serde's [serialize_with] attribute
/// <https://stackoverflow.com/questions/42723065/how-to-sort-hashmap-keys-when-serializing-with-serde>
fn ordered_i32_map<S, V: Serialize>(value: &HashMap<String, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<IntString, &V> = value.iter().map(|(k, v)| (IntString(k.clone()), v)).collect();
    ordered.serialize(serializer)
}

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

pub fn read_skel_component_gto_file(element_relpath: &str, data_dir: &str) -> BseSkelComponentGto {
    read_skel_component_gto_file_f(element_relpath, data_dir).unwrap()
}

pub fn read_skel_component_ecp_file(element_relpath: &str, data_dir: &str) -> BseSkelComponentEcp {
    read_skel_component_ecp_file_f(element_relpath, data_dir).unwrap()
}

pub fn read_skel_element_file(skel_element_relpath: &str, data_dir: &str) -> BseSkelElement {
    read_skel_element_file_f(skel_element_relpath, data_dir).unwrap()
}

pub fn read_skel_table_file(skel_table_relpath: &str, data_dir: &str) -> BseSkelTable {
    read_skel_table_file_f(skel_table_relpath, data_dir).unwrap()
}

pub fn read_skel_metadata_file(skel_metadata_relpath: &str, data_dir: &str) -> BseSkelMetadata {
    read_skel_metadata_file_f(skel_metadata_relpath, data_dir).unwrap()
}

#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{element_relpath}")} "#)]
pub fn read_skel_component_gto_file_f(element_relpath: &str, data_dir: &str) -> Result<BseSkelComponentGto, BseError> {
    let path = format!("{data_dir}/{element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{element_relpath}")} "#)]
pub fn read_skel_component_ecp_file_f(element_relpath: &str, data_dir: &str) -> Result<BseSkelComponentEcp, BseError> {
    let path = format!("{data_dir}/{element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_element_relpath}")} "#)]
pub fn read_skel_element_file_f(skel_element_relpath: &str, data_dir: &str) -> Result<BseSkelElement, BseError> {
    let path = format!("{data_dir}/{skel_element_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_table_relpath}")} "#)]
pub fn read_skel_table_file_f(skel_table_relpath: &str, data_dir: &str) -> Result<BseSkelTable, BseError> {
    let path = format!("{data_dir}/{skel_table_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
}

#[cached(size = 50, key = "String", convert = r#" {format!("{data_dir}/{skel_metadata_relpath}")} "#)]
pub fn read_skel_metadata_file_f(skel_metadata_relpath: &str, data_dir: &str) -> Result<BseSkelMetadata, BseError> {
    let path = format!("{data_dir}/{skel_metadata_relpath}");
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(BseError::from)
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
