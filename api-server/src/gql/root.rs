use async_graphql::connection::*;
use async_graphql::*;

use super::Dataset;

pub struct Query;

#[Object]
impl Query {
    /// Returns the version of the GQL API
    async fn api_version(&self) -> String {
        "1.0".to_string()
    }

    /// Returns dataset by its ID
    async fn dataset_by_id(&self, _ctx: &Context<'_>, id: String) -> Option<Dataset> {
        Some(Dataset { id: id })
    }

    /// Iterates all datasets
    async fn datasets(
        &self,
        _ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, Dataset>> {
        query(
            after,
            before,
            first,
            last,
            |_after, _before, _first, _last| async move {
                let mut connection = Connection::new(false, false);

                let datasets = vec![
                    Dataset {
                        id: "a".to_string(),
                    },
                    Dataset {
                        id: "b".to_string(),
                    },
                ];

                connection.append(datasets.into_iter().map(|ds| Edge::new(ds.id.clone(), ds)));

                Ok(connection)
            },
        )
        .await
    }
}

pub type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

pub fn schema() -> Schema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(extensions::ApolloTracing)
        .finish()
}
