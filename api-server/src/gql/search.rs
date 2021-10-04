use async_graphql::*;
use kamu::domain;

use super::{page_based_connection, Dataset};

///////////////////////////////////////////////////////////////////////////////
// Search
///////////////////////////////////////////////////////////////////////////////

pub(crate) struct Search;

#[Object]
impl Search {
    /// Perform search across all resources
    async fn query(
        &self,
        ctx: &Context<'_>,
        query: String,
        page: Option<usize>,
        #[graphql(default = 15)] per_page: usize,
    ) -> Result<SearchResultConnection> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();

        let page = page.unwrap_or(0);

        let nodes: Vec<_> = metadata_repo
            .get_all_datasets()
            .filter(|id| id.contains(&query))
            .skip(page * per_page)
            .take(per_page)
            .map(|id| SearchResult::Dataset(Dataset::new(id.into())))
            .collect();

        // TODO: Slow but temporary
        let total_count = metadata_repo.get_all_datasets().count();

        Ok(SearchResultConnection::new(
            nodes,
            page,
            per_page,
            Some(total_count),
        ))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Union, Debug, Clone)]
pub(crate) enum SearchResult {
    Dataset(Dataset),
    // Account,
    // Organization,
    // Issue,
}

///////////////////////////////////////////////////////////////////////////////

page_based_connection!(SearchResult, SearchResultConnection, SearchResultEdge);
