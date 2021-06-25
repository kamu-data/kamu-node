use async_graphql::connection::*;
use async_graphql::*;
use chrono::prelude::*;

#[derive(SimpleObject)]
pub(crate) struct MetadataBlock {
    block_hash: String,
    prev_block_hash: Option<String>,
    system_time: DateTime<Utc>,
    //pub output_slice: Option<DataSlice>,
    output_watermark: Option<DateTime<Utc>>,
    //pub input_slices: Option<Vec<DataSlice>>,
    //pub source: Option<DatasetSource>,
    //pub vocab: Option<DatasetVocabulary>,
}

pub(crate) struct MetadataChain {
    pub dataset_id: String,
}

#[Object]
impl MetadataChain {
    /// Returns a metadata block corresponding to the specified hash
    async fn block_by_hash(&self, _ctx: &Context<'_>, block_hash: String) -> Option<MetadataBlock> {
        Some(MetadataBlock {
            block_hash: block_hash,
            prev_block_hash: Some("aabbcc".to_owned()),
            system_time: Utc::now(),
            output_watermark: Some(Utc::now()),
        })
    }

    /// Iterates all metadata blocks starting from the latest one
    async fn blocks(
        &self,
        _ctx: &Context<'_>,
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

                let blocks = vec![
                    MetadataBlock {
                        block_hash: "bbccdd".to_owned(),
                        prev_block_hash: Some("aabbcc".to_owned()),
                        system_time: Utc::now(),
                        output_watermark: Some(Utc::now()),
                    },
                    MetadataBlock {
                        block_hash: "aabbcc".to_owned(),
                        prev_block_hash: None,
                        system_time: Utc::now(),
                        output_watermark: None,
                    },
                ];

                connection.append(
                    blocks
                        .into_iter()
                        .map(|b| Edge::new(b.block_hash.clone(), b)),
                );

                Ok(connection)
            },
        )
        .await
    }
}
