use async_graphql::*;
use chrono::prelude::*;
use kamu::domain;
use kamu::infra;

use super::*;

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Dataset {
    pub id: DatasetID,
}

#[ComplexObject]
impl Dataset {
    #[graphql(skip)]
    fn get_chain(&self, ctx: &Context<'_>) -> Result<Box<dyn domain::MetadataChain>> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
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
        let schema = query_svc.get_schema(&self.id)?;

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

    /// An estimated size of data on disk not accounting for replication or caching
    async fn data_size(&self, ctx: &Context<'_>) -> Result<u64> {
        let metadata_repo = from_catalog::<dyn domain::MetadataRepository>(ctx).unwrap();
        let summary = metadata_repo.get_summary(&self.id)?;
        Ok(summary.data_size)
    }

    /// Returns the specified number of the latest records in the dataset
    /// This is equivalent to the SQL query: `SELECT * FROM dataset ORDER BY event_time DESC LIMIT N`
    async fn tail(
        &self,
        ctx: &Context<'_>,
        num_records: Option<u64>,
        format: Option<DataSliceFormat>,
    ) -> Result<DataSlice> {
        use kamu::infra::utils::records_writers::*;

        let query_svc = from_catalog::<dyn domain::QueryService>(ctx).unwrap();
        let df = query_svc.tail(&self.id, num_records.unwrap_or(20))?;

        // TODO: Swithc to actix 3.x-beta or move to a different web server that is not lagging behind
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let records = runtime.block_on(df.collect())?;

        let mut buf = Vec::new();

        // TODO: Default to JsonSoA format once implemented
        let format = format.unwrap_or(DataSliceFormat::Json);

        {
            let mut writer: Box<dyn RecordsWriter> = match format {
                DataSliceFormat::Csv => {
                    Box::new(CsvWriterBuilder::new().has_headers(true).build(&mut buf))
                }
                DataSliceFormat::Json => Box::new(JsonArrayWriter::new(&mut buf)),
                DataSliceFormat::JsonLD => Box::new(JsonLineDelimitedWriter::new(&mut buf)),
                DataSliceFormat::JsonSoA => {
                    unimplemented!("SoA Json format is not yet implemented")
                }
            };

            writer.write_batches(&records)?;
            writer.finish()?;
        }

        Ok(DataSlice {
            format,
            content: String::from_utf8(buf).unwrap(),
        })
    }
}
