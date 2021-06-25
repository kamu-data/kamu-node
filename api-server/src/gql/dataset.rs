use async_graphql::*;

use super::MetadataChain;

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Dataset {
    pub id: String,
}

#[ComplexObject]
impl Dataset {
    /// Access to the temporal metadata of the dataset
    async fn metadata_chain(&self) -> MetadataChain {
        MetadataChain {
            dataset_id: self.id.clone(),
        }
    }
}
