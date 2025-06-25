//! Field definitions.

use crate::prelude::*;

/* #region field for components */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldGtoElectronShell {
    pub function_type: String,
    pub region: String,
    pub angular_momentum: Vec<i32>,
    #[serde(deserialize_with = "deserialize_string_as_f64_vec")]
    pub exponents: Vec<f64>,
    #[serde(deserialize_with = "deserialize_string_as_f64_vec_vec")]
    pub coefficients: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldGtoElement {
    pub references: Vec<String>,
    pub electron_shells: Vec<FieldGtoElectronShell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldEcpPotential {
    pub angular_momentum: Vec<i32>,
    #[serde(deserialize_with = "deserialize_string_as_f64_vec_vec")]
    pub coefficients: Vec<Vec<f64>>,
    pub ecp_type: String,
    pub r_exponents: Vec<i32>,
    #[serde(deserialize_with = "deserialize_string_as_f64_vec")]
    pub gaussian_exponents: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldEcpElement {
    pub references: Vec<String>,
    pub ecp_electrons: i32,
    pub ecp_potentials: Vec<FieldEcpPotential>,
}

/* #endregion */

/* #region field for skeletons */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldMolssiBseSchema {
    pub schema_type: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkelComponentGto {
    pub molssi_bse_schema: FieldMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    pub elements: HashMap<i32, FieldGtoElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkelComponentEcp {
    pub molssi_bse_schema: FieldMolssiBseSchema,
    pub description: String,
    pub data_source: String,
    pub elements: HashMap<i32, FieldEcpElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSkelElement {
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkelElement {
    pub molssi_bse_schema: FieldMolssiBseSchema,
    pub name: String,
    pub description: String,
    pub elements: HashMap<i32, FieldSkelElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkelTable {
    pub molssi_bse_schema: FieldMolssiBseSchema,
    pub revision_description: String,
    pub revision_date: String,
    pub elements: HashMap<i32, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkelMetadata {
    pub molssi_bse_schema: FieldMolssiBseSchema,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub family: String,
    pub description: String,
    pub role: String,
    pub auxiliaries: HashMap<String, String>,
}

/* #endregion */

/* #region ser/de of string f64 */

struct F64Visitor;
impl<'de> Visitor<'de> for F64Visitor {
    type Value = f64;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representation of a f64")
    }
    fn visit_str<E>(self, value: &str) -> Result<f64, E>
    where
        E: serde::de::Error,
    {
        value
            .parse::<f64>()
            .map_err(|_err| E::invalid_value(Unexpected::Str(value), &"a string representation of a f64"))
    }
}

pub fn deserialize_string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(F64Visitor)
}

struct VecF64Visitor;
impl<'de> Visitor<'de> for VecF64Visitor {
    type Value = Vec<f64>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of string representations of f64")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<f64>, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
        A::Error: serde::de::Error,
    {
        use serde::de::Error;
        let mut vec = Vec::new();
        while let Some(value) = seq.next_element::<String>()? {
            vec.push(value.parse::<f64>().map_err(|_err| {
                A::Error::invalid_value(Unexpected::Str(&value), &"a string representation of a f64")
            })?);
        }
        Ok(vec)
    }
}

pub fn deserialize_string_as_f64_vec<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(VecF64Visitor)
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

pub fn deserialize_string_as_f64_vec_vec<'de, D>(deserializer: D) -> Result<Vec<Vec<f64>>, D::Error>
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
        let bse_dir = env!("BSE_DEV_DIR");

        // skeleton components, gto
        let json_data =
            std::fs::read_to_string(format!("{bse_dir}/basis_set_exchange/data/ahlrichs/TZV/def2-TZVP-base.1.json"))
                .unwrap();
        let data: SkelComponentGto = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton components, ecp
        let json_data =
            std::fs::read_to_string(format!("{bse_dir}/basis_set_exchange/data/ahlrichs/ECP/def2-ECP.1.json")).unwrap();
        let data: SkelComponentEcp = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton elements
        let json_data =
            std::fs::read_to_string(format!("{bse_dir}/basis_set_exchange/data/ahlrichs/def2-QZVPP.1.element.json"))
                .unwrap();
        let data: SkelElement = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton table
        let json_data =
            std::fs::read_to_string(format!("{bse_dir}/basis_set_exchange/data/def2-QZVPP.1.table.json")).unwrap();
        let data: SkelTable = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");

        // skeleton metadata
        let json_data =
            std::fs::read_to_string(format!("{bse_dir}/basis_set_exchange/data/def2-QZVPP.metadata.json")).unwrap();
        let data: SkelMetadata = serde_json::from_str(&json_data).unwrap();
        println!("{data:?}");
    }
}
