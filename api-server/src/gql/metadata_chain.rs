use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use opendatafabric as odf;

use super::utils::from_catalog;
use super::{page_based_connection, DatasetID, Multihash};

////////////////////////////////////////////////////////////////////////////////////////
// MetadataBlock
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject, Debug, Clone)]
pub(crate) struct MetadataBlock {
    block_hash: Multihash,
    prev_block_hash: Option<Multihash>,
    system_time: DateTime<Utc>,
    //pub output_slice: Option<DataSlice>,
    output_watermark: Option<DateTime<Utc>>,
    //pub input_slices: Option<Vec<DataSlice>>,
    //pub source: Option<DatasetSource>,
    //pub vocab: Option<DatasetVocabulary>,
}

impl MetadataBlock {
    pub fn new(hash: odf::Multihash, block: odf::MetadataBlock) -> Self {
        Self {
            block_hash: hash.into(),
            prev_block_hash: block.prev_block_hash.map(|v| v.into()),
            system_time: block.system_time,
            output_watermark: block.output_watermark,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// MetadataRef
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject)]
pub(crate) struct BlockRef {
    name: String,
    block_hash: Multihash,
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
        Ok(metadata_repo.get_metadata_chain(&self.dataset_id.as_local_ref())?)
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
        hash: Multihash,
    ) -> Result<Option<MetadataBlock>> {
        let chain = self.get_chain(ctx)?;
        Ok(chain
            .get_block(&hash)
            .map(|b| MetadataBlock::new(hash.into(), b)))
    }

    // TODO: Add ref parameter (defaulting to "head")
    // TODO: Support before/after style iteration
    /// Iterates all metadata blocks in the reverse chronological order
    async fn blocks(
        &self,
        ctx: &Context<'_>,
        page: Option<usize>,
        #[graphql(default = 20)] per_page: usize,
    ) -> Result<MetadataBlockConnection> {
        let chain = self.get_chain(ctx)?;

        let page = page.unwrap_or(0);

        let nodes: Vec<_> = chain
            .iter_blocks()
            .skip(page * per_page)
            .take(per_page)
            .map(|(hash, block)| MetadataBlock::new(hash, block))
            .collect();

        // TODO: Slow but temporary
        let total_count = chain.iter_blocks().count();

        Ok(MetadataBlockConnection::new(
            nodes,
            page,
            per_page,
            Some(total_count),
        ))
    }
}

page_based_connection!(MetadataBlock, MetadataBlockConnection, MetadataBlockEdge);
