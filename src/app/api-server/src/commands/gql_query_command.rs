// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;
use kamu_accounts::CurrentAccountSubject;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct GqlQueryCommand {
    catalog: dill::Catalog,
    server_account_subject: CurrentAccountSubject,
    query: String,
    full: bool,
}

impl GqlQueryCommand {
    pub fn new(
        catalog: dill::Catalog,
        server_account_subject: CurrentAccountSubject,
        query: String,
        full: bool,
    ) -> Self {
        Self {
            catalog,
            server_account_subject,
            query,
            full,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl super::Command for GqlQueryCommand {
    async fn run(&mut self) -> Result<(), InternalError> {
        // TODO: Extract auth and TX handling outside of commands
        let catalog_with_auth = self
            .catalog
            .builder_chained()
            .add_value(self.server_account_subject.clone())
            .build();

        let txr = database_common::DatabaseTransactionRunner::new(catalog_with_auth);

        txr.transactional(|catalog| async move {
            let result = crate::gql_server::gql_query(&self.query, self.full, catalog).await;
            print!("{}", result);
            Ok(())
        })
        .await?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
