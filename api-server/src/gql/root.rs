use async_graphql::*;

use super::Datasets;
use super::Search;

pub struct Query;

#[Object]
impl Query {
    /// Returns the version of the GQL API
    async fn api_version(&self) -> String {
        "0.1".to_string()
    }

    /// Dataset-related functionality group
    async fn datasets(&self) -> Datasets {
        Datasets
    }

    /// Search-related functionality group
    async fn search(&self) -> Search {
        Search
    }
}

pub type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

pub fn schema(catalog: dill::Catalog) -> Schema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(extensions::ApolloTracing)
        .data(catalog)
        .finish()
}
