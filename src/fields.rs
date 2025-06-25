//! Field definitions.

use crate::prelude::*;

/* #region field for components */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldGtoElectronShell {
    pub function_type: String,
    pub region: String,
    pub angular_momentum: Vec<i32>,
    #[serde(deserialize_with = "deserialize_vec_f64")]
    pub exponents: Vec<f64>,
    #[serde(deserialize_with = "deserialize_vec_vec_f64")]
    pub coefficients: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldGtoElement {
    pub references: Vec<String>,
    pub electron_shells: Vec<BseFieldGtoElectronShell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldEcpPotential {
    pub angular_momentum: Vec<i32>,
    #[serde(deserialize_with = "deserialize_vec_vec_f64")]
    pub coefficients: Vec<Vec<f64>>,
    pub ecp_type: String,
    pub r_exponents: Vec<i32>,
    #[serde(deserialize_with = "deserialize_vec_f64")]
    pub gaussian_exponents: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldEcpElement {
    pub references: Vec<String>,
    pub ecp_electrons: i32,
    pub ecp_potentials: Vec<BseFieldEcpPotential>,
}

/* #endregion */

/* #region field for skeletons */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldMolssiBseSchema {
    pub schema_type: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentGto {
    pub molssi_bse_schema: BseFieldMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    pub elements: HashMap<i32, BseFieldGtoElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelComponentEcp {
    pub molssi_bse_schema: BseFieldMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    pub elements: HashMap<i32, BseFieldEcpElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseFieldSkelElement {
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelElement {
    pub molssi_bse_schema: BseFieldMolssiBseSchema,
    pub name: String,
    pub description: String,
    pub elements: HashMap<i32, BseFieldSkelElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelTable {
    pub molssi_bse_schema: BseFieldMolssiBseSchema,
    pub revision_description: String,
    pub revision_date: String,
    pub elements: HashMap<i32, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BseSkelMetadata {
    pub molssi_bse_schema: BseFieldMolssiBseSchema,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub family: String,
    pub description: String,
    pub role: String,
    #[serde(deserialize_with = "deserialize_auxiliary_map")]
    pub auxiliaries: HashMap<String, Vec<String>>,
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
    #[serde(deserialize_with = "deserialize_auxiliary_map")]
    pub auxiliaries: HashMap<String, Vec<String>>,
    pub versions: HashMap<i32, BseRootMetadataVer>,
}

/* #endregion */

/* #region ser/de implementation */

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FieldAuxiliary {
    Str(String),
    Vec(Vec<String>),
}

impl<'de> Deserialize<'de> for FieldAuxiliary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value: Value = Value::deserialize(deserializer)?;
        match value {
            Value::String(v) => Ok(FieldAuxiliary::Str(v)),
            Value::Array(arr) => Ok(FieldAuxiliary::Vec(arr.iter().map(|v| v.to_string()).collect())),
            _ => Err(D::Error::custom("Expected a string or an array of strings")),
        }
    }
}

pub fn deserialize_auxiliary_map<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let auxiliaries: HashMap<String, FieldAuxiliary> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for (key, value) in auxiliaries {
        match value {
            FieldAuxiliary::Str(s) => result.insert(key, vec![s]),
            FieldAuxiliary::Vec(v) => result.insert(key, v),
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
