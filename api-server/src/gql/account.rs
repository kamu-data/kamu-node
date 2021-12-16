use async_graphql::*;

use super::AccountID;

///////////////////////////////////////////////////////////////////////////////

#[derive(Interface, Debug, Clone)]
#[graphql(
    field(name = "id", type = "&AccountID"),
    field(name = "name", type = "&str")
)]
pub(crate) enum Account {
    User(User),
    Organization(Organization),
}

///////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub(crate) struct User {
    /// Unique identifier of this user account
    id: AccountID,
}

#[ComplexObject]
impl User {
    #[graphql(skip)]
    pub fn new(id: AccountID) -> Self {
        Self { id }
    }

    // TODO: UNMOCK
    /// Symbolic name
    async fn name(&self) -> &str {
        "anonymous"
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub(crate) struct Organization {
    /// Unique identifier of this organization account
    id: AccountID,
}

#[ComplexObject]
impl Organization {
    #[graphql(skip)]
    pub fn new(id: AccountID) -> Self {
        Self { id }
    }

    // TODO: UNMOCK
    /// Symbolic name
    async fn name(&self) -> &str {
        "anonymous"
    }
}
