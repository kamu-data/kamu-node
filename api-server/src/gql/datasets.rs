use std::{convert::TryFrom, ops::Deref};

use async_graphql::connection::*;
use async_graphql::*;
use kamu::domain;
use opendatafabric as odf;

use super::MetadataChain;

////////////////////////////////////////////////////////////////////////////////////////
// DatasetID
////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasetID(odf::DatasetIDBuf);

impl From<odf::DatasetIDBuf> for DatasetID {
    fn from(value: odf::DatasetIDBuf) -> Self {
        DatasetID(value)
    }
}

impl Into<odf::DatasetIDBuf> for DatasetID {
    fn into(self) -> odf::DatasetIDBuf {
        self.0
    }
}

impl Into<String> for DatasetID {
    fn into(self) -> String {
        self.0.into()
    }
}

impl Deref for DatasetID {
    type Target = odf::DatasetID;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[Scalar]
impl ScalarType for DatasetID {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let val = odf::DatasetIDBuf::try_from(value.as_str())?;
            Ok(val.into())
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Datasets
////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Datasets;

#[Object]
impl Datasets {
    /// Returns dataset by its ID
    async fn by_id(&self, ctx: &Context<'_>, id: DatasetID) -> Result<Option<Dataset>> {
        let cat = ctx.data::<dill::Catalog>().unwrap();
        let metadata_repo = cat.get_one::<dyn domain::MetadataRepository>().unwrap();
        match metadata_repo.get_metadata_chain(&id) {
            Ok(_) => Ok(Some(Dataset { id })),
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
                        .map(|id| Edge::new(id.to_string(), Dataset { id: id.into() })),
                );

                Ok(connection)
            },
        )
        .await
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Dataset
////////////////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Dataset {
    pub id: DatasetID,
}

#[ComplexObject]
impl Dataset {
    /// Access to the temporal metadata of the dataset
    async fn metadata_chain(&self) -> MetadataChain {
        MetadataChain::new(self.id.clone())
    }
}
