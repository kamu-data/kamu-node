use std::{convert::TryFrom, ops::Deref};

use async_graphql::connection::*;
use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use opendatafabric as odf;

use super::MetadataChain;

////////////////////////////////////////////////////////////////////////////////////////
// DatasetID
////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////
// Datasets
////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Datasets;

#[Object]
impl Datasets {
    /// Returns dataset by its ID
    async fn by_id(&self, ctx: &Context<'_>, id: DatasetID) -> Result<Option<Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        match metadata_repo.get_metadata_chain(&id) {
            Ok(_) => Ok(Some(Dataset { id })),
            Err(domain::DomainError::DoesNotExist { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // TODO: Should be per-account
    /// Iterates all datasets
    async fn all(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        query(
            after,
            before,
            first,
            last,
            |_after, _before, _first, _last| async move {
                let mut connection = Connection::new(false, false);

                // TODO: proper iteration
                connection.append(
                    metadata_repo
                        .get_all_datasets()
                        .map(|id| Edge::new(id.to_string(), Dataset { id: id.into() })),
                );

                Ok(connection)
            },
        )
        .await
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Dataset
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Dataset {
    pub id: DatasetID,
}

#[ComplexObject]
impl Dataset {
    #[graphql(skip)]
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn domain::MetadataChain>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        Ok(metadata_repo.get_metadata_chain(&self.id)?)
    }

    /// Access to the temporal metadata of the dataset
    async fn metadata_chain(&self) -> MetadataChain {
        MetadataChain::new(self.id.clone())
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

    /// Total number of output records in this dataset
    async fn num_records_total(&self, ctx: &Context<'_>) -> Result<u64> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        let summary = metadata_repo.get_summary(&self.id)?;
        Ok(summary.num_records)
    }

    /// Records with event time prior to the watermark have already been reflected in the dataset (with high degree of certainty, but not guaranteed)
    async fn last_watermark(&self, ctx: &Context<'_>) -> Result<Option<DateTime<Utc>>> {
        let chain = self.get_chain(ctx)?;
        Ok(chain
            .iter_blocks_ref(&domain::BlockRef::Head)
            .filter_map(|b| b.output_watermark)
            .next())
    }

    /// An estimated size of data on disk not accounting for replication or caching
    async fn data_size(&self, ctx: &Context<'_>) -> Result<u64> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        let summary = metadata_repo.get_summary(&self.id)?;
        Ok(summary.data_size)
    }
}
