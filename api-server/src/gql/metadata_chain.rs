use std::convert::TryFrom;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use async_graphql::connection::*;
use async_graphql::*;
use chrono::prelude::*;
use kamu::domain as dom;
use opendatafabric as odf;

use super::DatasetID;

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

////////////////////////////////////////////////////////////////////////////////////////
// MetadataBlock
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject)]
pub(crate) struct MetadataBlock {
    block_hash: Sha3_256,
    prev_block_hash: Option<Sha3_256>,
    system_time: DateTime<Utc>,
    //pub output_slice: Option<DataSlice>,
    output_watermark: Option<DateTime<Utc>>,
    //pub input_slices: Option<Vec<DataSlice>>,
    //pub source: Option<DatasetSource>,
    //pub vocab: Option<DatasetVocabulary>,
}

impl From<odf::MetadataBlock> for MetadataBlock {
    fn from(val: odf::MetadataBlock) -> Self {
        Self {
            block_hash: val.block_hash.into(),
            prev_block_hash: val.prev_block_hash.map(|v| v.into()),
            system_time: val.system_time,
            output_watermark: val.output_watermark,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// MetadataChain
////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct MetadataChain {
    dataset_id: DatasetID,
}

#[Object]
impl MetadataChain {
    #[graphql(skip)]
    pub fn new(dataset_id: DatasetID) -> Self {
        Self { dataset_id }
    }

    #[graphql(skip)]
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn dom::MetadataChain>> {
        let repo_ref = ctx
            .data::<Arc<Mutex<dyn dom::MetadataRepository>>>()
            .unwrap();
        let repo = repo_ref.lock().unwrap();
        Ok(repo.get_metadata_chain(&self.dataset_id)?)
    }

    /// Returns a metadata block corresponding to the specified hash
    async fn block_by_hash(
        &self,
        ctx: &Context<'_>,
        hash: Sha3_256,
    ) -> Result<Option<MetadataBlock>> {
        let chain = self.get_chain(ctx)?;
        Ok(chain.get_block(&hash).map(|b| b.into()))
    }

    // TODO: Add ref parameter (defaulting to "head")
    /// Iterates all metadata blocks starting from the latest one
    async fn blocks(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, MetadataBlock>> {
        query(
            after,
            before,
            first,
            last,
            |_after, _before, _first, _last| async move {
                let mut connection = Connection::new(false, false);

                let chain = self.get_chain(ctx)?;

                connection.append(
                    chain
                        .iter_blocks()
                        .map(|b| Edge::new(b.block_hash.to_string(), b.into())),
                );

                Ok(connection)
            },
        )
        .await
    }
}
