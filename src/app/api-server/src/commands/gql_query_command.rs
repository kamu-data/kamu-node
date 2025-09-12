// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;

use super::{Command, CommandDesc};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[dill::component]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc {
    needs_admin_auth: true,
    needs_transaction: true,
})]
pub struct GqlQueryCommand {
    catalog: dill::Catalog,

    #[dill::component(explicit)]
    query: String,

    #[dill::component(explicit)]
    full: bool,
}

#[async_trait::async_trait]
impl Command for GqlQueryCommand {
    async fn run(&self) -> Result<(), InternalError> {
        let result =
            crate::gql_server::gql_query(&self.query, self.full, self.catalog.clone()).await;
        print!("{result}");
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
