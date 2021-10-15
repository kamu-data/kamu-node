use std::{convert::TryFrom, ops::Deref};

use async_graphql::*;
use opendatafabric as odf;

///////////////////////////////////////////////////////////////////////////////
// Page-based connection
///////////////////////////////////////////////////////////////////////////////

macro_rules! page_based_connection {
    ($node_type:ident, $connection_type:ident, $edge_type:ident) => {
        #[derive(SimpleObject)]
        #[graphql(complex)]
        pub(crate) struct $connection_type {
            /// A shorthand for `edges { node { ... } }`
            pub nodes: Vec<$node_type>,

            /// Approximate number of total nodes
            pub total_count: Option<usize>,

            /// Page information
            pub page_info: crate::gql::types::PageBasedInfo,
        }

        #[ComplexObject]
        impl $connection_type {
            #[graphql(skip)]
            pub fn new(
                nodes: Vec<$node_type>,
                page: usize,
                per_page: usize,
                total_count: Option<usize>,
            ) -> Self {
                let (total_pages, has_next_page) = match total_count {
                    None => (None, nodes.len() != per_page),
                    Some(0) => (Some(0), false),
                    Some(tc) => (
                        Some(tc.div_ceil(per_page)),
                        (tc.div_ceil(per_page) - 1) > page,
                    ),
                };

                Self {
                    nodes,
                    total_count,
                    page_info: crate::gql::types::PageBasedInfo {
                        has_previous_page: page > 0,
                        has_next_page,
                        total_pages,
                    },
                }
            }
            async fn edges(&self) -> Vec<$edge_type> {
                self.nodes
                    .iter()
                    .map(|node| $edge_type { node: node.clone() })
                    .collect()
            }
        }

        #[derive(SimpleObject)]
        pub(crate) struct $edge_type {
            pub node: $node_type,
        }
    };
}

pub(crate) use page_based_connection;

#[derive(SimpleObject)]
pub struct PageBasedInfo {
    /// When paginating backwards, are there more items?
    pub has_previous_page: bool,

    /// When paginating forwards, are there more items?
    pub has_next_page: bool,

    /// Approximate number of total pages assuming number of nodes per page stays the same
    pub total_pages: Option<usize>,
}

////////////////////////////////////////////////////////////////////////////////////////
// SHA
////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
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
// AccountID
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AccountID(odf::UsernameBuf);

impl From<odf::UsernameBuf> for AccountID {
    fn from(value: odf::UsernameBuf) -> Self {
        AccountID(value)
    }
}

impl Into<odf::UsernameBuf> for AccountID {
    fn into(self) -> odf::UsernameBuf {
        self.0
    }
}

impl Into<String> for AccountID {
    fn into(self) -> String {
        self.0.into()
    }
}

impl Deref for AccountID {
    type Target = odf::Username;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[Scalar]
impl ScalarType for AccountID {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let val = odf::UsernameBuf::try_from(value.as_str())?;
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
