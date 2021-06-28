use std::sync::Arc;
use std::sync::Mutex;

use async_graphql::connection::*;
use async_graphql::*;

use kamu::domain::*;

use super::Dataset;
use super::DatasetID;

pub struct Query;

#[Object]
impl Query {
    /// Returns the version of the GQL API
    async fn api_version(&self) -> String {
        "1.0".to_string()
    }

    /// Returns dataset by its ID
    async fn dataset_by_id(&self, ctx: &Context<'_>, id: DatasetID) -> Result<Option<Dataset>> {
        let metadata_repo_c = ctx.data::<Arc<Mutex<dyn MetadataRepository>>>().unwrap();
        let metadata_repo = metadata_repo_c.lock().unwrap();
        match metadata_repo.get_metadata_chain(&id) {
            Ok(_) => Ok(Some(Dataset { id })),
            Err(DomainError::DoesNotExist { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // TODO: Should be per-user
    /// Iterates all datasets
    async fn datasets(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, Dataset>> {
        let metadata_repo_c = ctx.data::<Arc<Mutex<dyn MetadataRepository>>>().unwrap();
        query(
            after,
            before,
            first,
            last,
            |_after, _before, _first, _last| async move {
                let metadata_repo = metadata_repo_c.lock().unwrap();

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

pub type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

pub fn schema(metadata_repo: Arc<Mutex<dyn MetadataRepository>>) -> Schema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(extensions::ApolloTracing)
        .data(metadata_repo)
        .finish()
}
