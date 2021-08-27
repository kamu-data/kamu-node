use std::sync::Arc;

use async_graphql::Context;

pub(crate) fn from_catalog<T>(ctx: &Context<'_>) -> Result<Arc<T>, dill::InjectionError>
where
    T: ?Sized + Send + Sync + 'static,
{
    let cat = ctx.data::<dill::Catalog>().unwrap();
    cat.get_one::<T>()
}
