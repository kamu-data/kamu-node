use async_graphql::*;

use super::*;

///////////////////////////////////////////////////////////////////////////////

pub(crate) struct Accounts;

#[Object]
impl Accounts {
    /// Returns account by its ID
    async fn by_id(&self, _ctx: &Context<'_>, account_id: AccountID) -> Result<Option<Account>> {
        Ok(Some(Account::User(User::new(account_id))))
    }

    /// Returns account by its name
    async fn by_name(&self, _ctx: &Context<'_>, name: String) -> Result<Option<Account>> {
        Ok(Some(Account::User(User::new(AccountID::mock()))))
    }
}

///////////////////////////////////////////////////////////////////////////////
