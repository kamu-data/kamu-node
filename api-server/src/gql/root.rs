use async_graphql::*;

use super::Accounts;
use super::Auth;
use super::Datasets;
use super::Search;

////////////////////////////////////////////////////////////////////////////////////////
// Query
////////////////////////////////////////////////////////////////////////////////////////

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

    /// Account-related functionality group
    async fn accounts(&self) -> Accounts {
        Accounts
    }

    /// Search-related functionality group
    async fn search(&self) -> Search {
        Search
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Mutation
////////////////////////////////////////////////////////////////////////////////////////

pub struct Mutation;

#[Object]
impl Mutation {
    async fn auth(&self) -> Auth {
        Auth
    }
}

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub fn schema(catalog: dill::Catalog) -> Schema {
    Schema::build(Query, Mutation, EmptySubscription)
        .extension(extensions::ApolloTracing)
        .data(catalog)
        .finish()
}
