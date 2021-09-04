use std::{convert::TryFrom, ops::Deref};

use async_graphql::*;
use opendatafabric as odf;

////////////////////////////////////////////////////////////////////////////////////////
// SHA
////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Sha3_256(odf::Sha3_256);

impl From<odf::Sha3_256> for Sha3_256 {
    fn from(value: odf::Sha3_256) -> Self {
        Sha3_256(value)
    }
}

impl Into<odf::Sha3_256> for Sha3_256 {
    fn into(self) -> odf::Sha3_256 {
        self.0
    }
}

impl Deref for Sha3_256 {
    type Target = odf::Sha3_256;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[Scalar]
impl ScalarType for Sha3_256 {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let sha = odf::Sha3_256::try_from(value.as_str())?;
            Ok(sha.into())
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// DatasetID
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasetID(odf::DatasetIDBuf);

impl From<odf::DatasetIDBuf> for DatasetID {
    fn from(value: odf::DatasetIDBuf) -> Self {
        DatasetID(value)
    }
}

impl Into<odf::DatasetIDBuf> for DatasetID {
    fn into(self) -> odf::DatasetIDBuf {
        self.0
    }
}

impl Into<String> for DatasetID {
    fn into(self) -> String {
        self.0.into()
    }
}

impl Deref for DatasetID {
    type Target = odf::DatasetID;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[Scalar]
impl ScalarType for DatasetID {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let val = odf::DatasetIDBuf::try_from(value.as_str())?;
            Ok(val.into())
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// DatasetKind
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatasetKind {
    Root,
    Derivative,
}

/////////////////////////////////////////////////////////////////////////////////////////
// DataSchema
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSchemaFormat {
    Parquet,
    ParquetJson,
}

#[derive(SimpleObject)]
pub(crate) struct DataSchema {
    pub format: DataSchemaFormat,
    pub content: String,
}

/////////////////////////////////////////////////////////////////////////////////////////
// DataSlice
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSliceFormat {
    Json,
    JsonLD,
    JsonSoA,
    Csv,
}

#[derive(SimpleObject)]
pub(crate) struct DataSlice {
    pub format: DataSliceFormat,
    pub content: String,
}
