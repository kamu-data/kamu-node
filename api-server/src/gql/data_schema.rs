use async_graphql::*;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSchemaFormat {
    Parquet,
    ParquetJson,
}

#[derive(SimpleObject)]
pub(crate) struct DataSchema {
    pub format: DataSchemaFormat,
    // TODO: should be bytes?
    pub content: String,
}
