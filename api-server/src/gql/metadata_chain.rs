use std::convert::TryFrom;

use async_graphql::connection::*;
use async_graphql::*;
use chrono::prelude::*;
use opendatafabric as odf;

struct Sha3_256(odf::Sha3_256);

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

#[Scalar]
impl ScalarType for Sha3_256 {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            // Parse the integer value
            let sha = odf::Sha3_256::try_from(value.as_str())?;
            Ok(sha.into())
        } else {
            // If the type does not match
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

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

pub(crate) struct MetadataChain {
    pub dataset_id: String,
}

#[Object]
impl MetadataChain {
    /// Returns a metadata block corresponding to the specified hash
    async fn block_by_hash(&self, _ctx: &Context<'_>, hash: Sha3_256) -> Option<MetadataBlock> {
        Some(MetadataBlock {
            block_hash: hash,
            prev_block_hash: Some(odf::Sha3_256::zero().into()),
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
                        block_hash: odf::Sha3_256::zero().into(),
                        prev_block_hash: Some(odf::Sha3_256::zero().into()),
                        system_time: Utc::now(),
                        output_watermark: Some(Utc::now()),
                    },
                    MetadataBlock {
                        block_hash: odf::Sha3_256::zero().into(),
                        prev_block_hash: None,
                        system_time: Utc::now(),
                        output_watermark: None,
                    },
                ];

                connection.append(
                    blocks
                        .into_iter()
                        .map(|b| Edge::new(b.block_hash.0.to_string(), b)),
                );

                Ok(connection)
            },
        )
        .await
    }
}
