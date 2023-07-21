use fvm_ipld_encoding::strict_bytes;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use rusqlite::types::Value;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DeserializeAs, SerializeAs};

pub const SQLITE_PAGE_SIZE: usize = 4096;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Execute = 2,
    Query = 3,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    #[serde(with = "strict_bytes")]
    pub db: Vec<u8>,
    pub buck_size: usize,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct ExecuteParams {
    pub stmts: Vec<String>,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct ExecuteReturn {
    pub effected_rows: usize,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct QueryParams {
    pub stmt: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryReturn {
    pub cols: Vec<String>,
    #[serde_as(as = "Vec<Vec<ValueDef>>")]
    pub rows: Vec<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "Value", untagged)]
pub enum ValueDef {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SerializeAs<Value> for ValueDef {
    fn serialize_as<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ValueDef::serialize(value, serializer)
    }
}

impl<'de> DeserializeAs<'de, Value> for ValueDef {
    fn deserialize_as<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ValueDef::deserialize(deserializer)
    }
}