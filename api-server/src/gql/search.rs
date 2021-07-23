use async_graphql::connection::*;
use async_graphql::*;
use kamu::domain;

use super::Dataset;

////////////////////////////////////////////////////////////////////////////////////////
// Search
////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Search;

#[Object]
impl Search {
    /// Perform search across all resources
    async fn query(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
        query: String,
    ) -> Result<Connection<String, SearchQueryResult>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        async_graphql::connection::query(
            after,
            before,
            first,
            last,
            |_after, _before, _first, _last| async move {
                let mut connection = Connection::new(false, false);

                // TODO: proper iteration
                connection.append(
                    metadata_repo
                        .get_all_datasets()
                        .filter(|id| id.contains(&query))
                        .map(|id| {
                            Edge::new(
                                id.to_string(),
                                SearchQueryResult::Dataset(Dataset { id: id.into() }),
                            )
                        }),
                );

                Ok(connection)
            },
        )
        .await
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// SearchQueryResult
////////////////////////////////////////////////////////////////////////////////////////

#[derive(Union)]
pub(crate) enum SearchQueryResult {
    Dataset(Dataset),
    // Account
    // Organization
    // Issue
}
