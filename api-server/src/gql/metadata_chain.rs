use async_graphql::connection::*;
use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use opendatafabric as odf;

use super::utils::from_catalog;
use super::{DatasetID, Sha3_256};

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
// MetadataRef
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject)]
pub(crate) struct BlockRef {
    name: String,
    block_hash: Sha3_256,
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
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn domain::MetadataChain>> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        Ok(metadata_repo.get_metadata_chain(&self.dataset_id)?)
    }

    /// Returns all named metadata block references
    async fn refs(&self, ctx: &Context<'_>) -> Result<Vec<BlockRef>> {
        let chain = self.get_chain(ctx)?;
        Ok(vec![BlockRef {
            name: "head".to_owned(),
            block_hash: chain.read_ref(&domain::BlockRef::Head).unwrap().into(),
        }])
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
