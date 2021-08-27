use async_graphql::connection::*;
use async_graphql::*;
use kamu::domain;

use super::*;

pub(crate) struct Datasets;

#[Object]
impl Datasets {
    /// Returns dataset by its ID
    async fn by_id(&self, ctx: &Context<'_>, id: DatasetID) -> Result<Option<Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        match metadata_repo.get_metadata_chain(&id) {
            Ok(_) => Ok(Some(Dataset::new(id))),
            Err(domain::DomainError::DoesNotExist { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // TODO: Should be per-account
    /// Iterates all datasets
    async fn all(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        query(
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
                        .map(|id| Edge::new(id.to_string(), Dataset::new(id.into()))),
                );

                Ok(connection)
            },
        )
        .await
    }
}
