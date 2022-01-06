use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use kamu::infra;
use opendatafabric as odf;
use opendatafabric::IntoDataStreamBlock;

use super::*;

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct DatasetMetadata {
    pub dataset_id: DatasetID,
}

#[ComplexObject]
impl DatasetMetadata {
    #[graphql(skip)]
    pub fn new(dataset_id: DatasetID) -> Self {
        Self { dataset_id }
    }

    #[graphql(skip)]
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn domain::MetadataChain>> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        Ok(metadata_repo.get_metadata_chain(&self.dataset_id.as_local_ref())?)
    }

    /// Access to the temporal metadata chain of the dataset
    async fn chain(&self) -> MetadataChain {
        MetadataChain::new(self.dataset_id.clone())
    }

    /// Last recorded watermark
    async fn current_watermark(&self, ctx: &Context<'_>) -> Result<Option<DateTime<Utc>>> {
        let chain = self.get_chain(ctx)?;
        Ok(chain
            .iter_blocks_ref(&domain::BlockRef::Head)
            .filter_map(|(_, b)| b.into_data_stream_block())
            .find_map(|b| b.event.output_watermark))
    }

    /// Latest data schema
    async fn current_schema(
        &self,
        ctx: &Context<'_>,
        format: Option<DataSchemaFormat>,
    ) -> Result<DataSchema> {
        let query_svc = from_catalog::<dyn domain::QueryService>(ctx).unwrap();
        let schema = query_svc.get_schema(&self.dataset_id.as_local_ref())?;

        let format = format.unwrap_or(DataSchemaFormat::Parquet);
        let mut buf = Vec::new();

        match format {
            DataSchemaFormat::Parquet => {
                infra::utils::schema_utils::write_schema_parquet(&mut buf, &schema)
            }
            DataSchemaFormat::ParquetJson => {
                infra::utils::schema_utils::write_schema_parquet_json(&mut buf, &schema)
            }
        }
        .unwrap();

        Ok(DataSchema {
            format,
            content: String::from_utf8(buf).unwrap(),
        })
    }

    /// Current upstream dependencies of a dataset
    async fn current_upstream_dependencies(&self, ctx: &Context<'_>) -> Result<Vec<Dataset>> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        let summary = metadata_repo.get_summary(&self.dataset_id.as_local_ref())?;
        Ok(summary
            .dependencies
            .into_iter()
            .map(|i| Dataset::new(AccountID::mock(), i.id.unwrap().into()))
            .collect())
    }

    /// Current downstream dependencies of a dataset
    async fn current_downstream_dependencies(&self, ctx: &Context<'_>) -> Result<Vec<Dataset>> {
        let dataset_id: odf::DatasetID = self.dataset_id.clone().into();
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();

        // TODO: This is really slow
        Ok(metadata_repo
            .get_all_datasets()
            .filter(|hdl| hdl.id != dataset_id)
            .map(|hdl| metadata_repo.get_summary(&hdl.as_local_ref()).unwrap())
            .filter(|sum| {
                sum.dependencies
                    .iter()
                    .any(|i| i.id.as_ref() == Some(&dataset_id))
            })
            .map(|sum| sum.id)
            .map(|id| Dataset::new(AccountID::mock(), id.into()))
            .collect())
    }
}
