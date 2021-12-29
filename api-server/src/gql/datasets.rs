use async_graphql::*;
use kamu::domain;

use super::*;

///////////////////////////////////////////////////////////////////////////////

pub(crate) struct Datasets;

#[Object]
impl Datasets {
    /// Returns dataset by its ID
    async fn by_id(&self, ctx: &Context<'_>, dataset_id: DatasetID) -> Result<Option<Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        match metadata_repo.get_metadata_chain(&dataset_id.as_local_ref()) {
            Ok(_) => Ok(Some(Dataset::new(AccountID::mock(), dataset_id))),
            Err(domain::DomainError::DoesNotExist { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // TODO: Multitenancy
    /// Returns datasets belonging to the specified account
    async fn by_account_id(
        &self,
        ctx: &Context<'_>,
        account_id: AccountID,
        page: Option<usize>,
        #[graphql(default = 15)] per_page: usize,
    ) -> Result<DatasetConnection> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();

        let page = page.unwrap_or(0);

        let nodes: Vec<_> = metadata_repo
            .get_all_datasets()
            .skip(page * per_page)
            .take(per_page)
            .map(|hdl| Dataset::new(account_id.clone(), hdl.id.into()))
            .collect();

        // TODO: Slow but temporary
        let total_count = metadata_repo.get_all_datasets().count();

        Ok(DatasetConnection::new(
            nodes,
            page,
            per_page,
            Some(total_count),
        ))
    }
}

///////////////////////////////////////////////////////////////////////////////

page_based_connection!(Dataset, DatasetConnection, DatasetEdge);
