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
    #[serde(deserialize_with = "deserialize_vec_f64")]
    pub exponents: Vec<f64>,
    #[serde(deserialize_with = "deserialize_vec_vec_f64")]
    pub coefficients: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseGtoElement {
    pub references: Vec<String>,
    pub electron_shells: Vec<BseElectronShell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseEcpPotential {
    pub angular_momentum: Vec<i32>,
    #[serde(deserialize_with = "deserialize_vec_vec_f64")]
    pub coefficients: Vec<Vec<f64>>,
    pub ecp_type: String,
    pub r_exponents: Vec<i32>,
    #[serde(deserialize_with = "deserialize_vec_f64")]
    pub gaussian_exponents: Vec<f64>,
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
    #[serde(serialize_with = "ordered_map")]
    pub elements: HashMap<i32, BseGtoElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentEcp {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    #[serde(serialize_with = "ordered_map")]
    pub elements: HashMap<i32, BseEcpElement>,
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
    #[serde(serialize_with = "ordered_map")]
    pub elements: HashMap<i32, BseElementComponents>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelTable {
    pub molssi_bse_schema: BseMolssiBseSchema,
    pub revision_description: String,
    pub revision_date: String,
    #[serde(serialize_with = "ordered_map")]
    pub elements: HashMap<i32, String>,
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
    #[serde(deserialize_with = "deserialize_vec_i32")]
    pub elements: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseRootMetadata {
    pub display_name: String,
    pub other_names: Vec<String>,
    pub description: String,
    #[serde(deserialize_with = "deserialize_i32")]
    pub latest_version: i32,
    pub tags: Vec<String>,
    pub basename: String,
    pub relpath: String,
    pub family: String,
    pub role: String,
    pub function_types: Vec<String>,
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    #[serde(serialize_with = "ordered_map")]
    pub versions: HashMap<i32, BseRootMetadataVer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseBasisReference {
    pub reference_keys: Vec<String>,
    pub reference_description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseBasisElement {
    pub references: Vec<BseBasisReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub electron_shells: Option<Vec<BseElectronShell>>,
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
    pub version: i32,
    pub function_types: Vec<String>,
    pub name: String,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub family: String,
    pub description: String,
    pub role: String,
    #[serde(serialize_with = "ordered_map")]
    pub auxiliaries: HashMap<String, BseAuxiliary>,
    #[serde(serialize_with = "ordered_map")]
    pub elements: HashMap<i32, BseBasisElement>,
}

/* #endregion */

/* #region ser/de implementation */

/// For use with serde's [serialize_with] attribute
/// <https://stackoverflow.com/questions/42723065/how-to-sort-hashmap-keys-when-serializing-with-serde>
fn ordered_map<S, K: Ord + Serialize, V: Serialize>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
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

struct F64Visitor;
struct I32Visitor;

#[duplicate_item(
    T     TVistor      info                               ;
   [f64] [F64Visitor] ["a string representation of a f64"];
   [i32] [I32Visitor] ["a string representation of a i32"];
)]
impl<'de> Visitor<'de> for TVistor {
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(info)
    }

    fn visit_str<E>(self, value: &str) -> Result<T, E>
    where
        E: serde::de::Error,
    {
        value.parse::<T>().map_err(|_err| E::invalid_value(Unexpected::Str(value), &info))
    }
}

#[duplicate_item(
    T     TVistor      deserialize_ty ;
   [f64] [F64Visitor] [deserialize_f64];
   [i32] [I32Visitor] [deserialize_i32];
)]
pub fn deserialize_ty<'de, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(TVistor)
}

struct VecF64Visitor;
struct VecI32Visitor;

#[duplicate_item(
    T     TVistor         info                                           ;
   [f64] [VecF64Visitor] ["a sequence of string representation of a f64"];
   [i32] [VecI32Visitor] ["a sequence of string representation of a i32"];
)]
impl<'de> Visitor<'de> for TVistor {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(info)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
        A::Error: serde::de::Error,
    {
        use serde::de::Error;
        let mut vec = Vec::new();
        while let Some(value) = seq.next_element::<String>()? {
            vec.push(value.parse::<T>().map_err(|_err| A::Error::invalid_value(Unexpected::Str(&value), &info))?);
        }
        Ok(vec)
    }
}

#[duplicate_item(
    T     TVistor         deserialize_ty      ;
   [f64] [VecF64Visitor] [deserialize_vec_f64];
   [i32] [VecI32Visitor] [deserialize_vec_i32];
)]
pub fn deserialize_ty<'de, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(TVistor)
}

struct VecVecF64Visitor;

impl<'de> Visitor<'de> for VecVecF64Visitor {
    type Value = Vec<Vec<f64>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of sequences of string representations of f64")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<Vec<f64>>, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
        A::Error: serde::de::Error,
    {
        use serde::de::Error;
        let mut vec = Vec::new();
        while let Some(inner_seq) = seq.next_element::<Vec<String>>()? {
            let inner_vec = inner_seq
                .into_iter()
                .map(|s| {
                    s.parse::<f64>().map_err(|_err| {
                        A::Error::invalid_value(Unexpected::Str(&s), &"a string representation of a f64")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            vec.push(inner_vec);
        }
        Ok(vec)
    }
}

pub fn deserialize_vec_vec_f64<'de, D>(deserializer: D) -> Result<Vec<Vec<f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(VecVecF64Visitor)
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
