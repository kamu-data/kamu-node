use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use kamu::infra;

use super::*;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub(crate) struct Dataset {
    dataset_id: DatasetID,
}

#[ComplexObject]
impl Dataset {
    #[graphql(skip)]
    pub fn new(dataset_id: DatasetID) -> Self {
        Self { dataset_id }
    }

    #[graphql(skip)]
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn domain::MetadataChain>> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        Ok(metadata_repo.get_metadata_chain(&self.dataset_id)?)
    }

    async fn id(&self) -> DatasetID {
        self.dataset_id.clone()
    }

    /// Returns the kind of a dataset (Root or Derivative)
    async fn kind(&self, ctx: &Context<'_>) -> Result<DatasetKind> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        let summary = metadata_repo.get_summary(&self.dataset_id)?;
        Ok(match summary.kind {
            infra::DatasetKind::Root => DatasetKind::Root,
            infra::DatasetKind::Derivative => DatasetKind::Derivative,
        })
    }

    /// Access to the data of the dataset
    async fn data(&self) -> DatasetData {
        DatasetData::new(self.dataset_id.clone())
    }

    /// Access to the metadata of the dataset
    async fn metadata(&self) -> DatasetMetadata {
        DatasetMetadata::new(self.dataset_id.clone())
    }

    // TODO: Performance
    /// Creation time of the first metadata block in the chain
    async fn created_at(&self, ctx: &Context<'_>) -> Result<DateTime<Utc>> {
        let chain = self.get_chain(ctx)?;
        let first_block = chain
            .iter_blocks_ref(&domain::BlockRef::Head)
            .last()
            .expect("Dataset without blocks");
        Ok(first_block.system_time)
    }

    /// Creation time of the most recent metadata block in the chain
    async fn last_updated_at(&self, ctx: &Context<'_>) -> Result<DateTime<Utc>> {
        let chain = self.get_chain(ctx)?;
        let last_block = chain
            .read_ref(&domain::BlockRef::Head)
            .expect("Dataset without blocks");
        let block = chain.get_block(&last_block).expect("Can't read block");
        Ok(block.system_time)
    }
}
