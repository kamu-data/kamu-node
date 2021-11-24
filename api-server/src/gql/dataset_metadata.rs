use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use kamu::infra;
use opendatafabric as odf;

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
        Ok(metadata_repo.get_metadata_chain(&self.dataset_id)?)
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
            .filter_map(|b| b.output_watermark)
            .next())
    }

    /// Latest data schema
    async fn current_schema(
        &self,
        ctx: &Context<'_>,
        format: Option<DataSchemaFormat>,
    ) -> Result<DataSchema> {
        let query_svc = from_catalog::<dyn domain::QueryService>(ctx).unwrap();
        let schema = query_svc.get_schema(&self.dataset_id)?;

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
        let summary = metadata_repo.get_summary(&self.dataset_id)?;
        Ok(summary
            .dependencies
            .into_iter()
            .map(|id| Dataset::new(id.into()))
            .collect())
    }

    /// Current downstream dependencies of a dataset
    async fn current_downstream_dependencies(&self, ctx: &Context<'_>) -> Result<Vec<Dataset>> {
        let dataset_id: odf::DatasetIDBuf = self.dataset_id.clone().into();
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();

        // TODO: This is really slow
        Ok(metadata_repo
            .get_all_datasets()
            .filter(|id| *id != dataset_id)
            .map(|id| metadata_repo.get_summary(&id).unwrap())
            .filter(|sum| sum.dependencies.contains(&dataset_id))
            .map(|sum| sum.id)
            .map(|id| Dataset::new(id.into()))
            .collect())
    }
}
