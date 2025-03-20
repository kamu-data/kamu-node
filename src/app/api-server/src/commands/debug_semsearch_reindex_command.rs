// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use init_on_startup::InitOnStartup as _;
use internal_error::*;
use kamu_accounts::CurrentAccountSubject;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DebugSemsearchReindexCommand {
    catalog: dill::Catalog,
    server_account_subject: CurrentAccountSubject,
}

impl DebugSemsearchReindexCommand {
    pub fn new(catalog: dill::Catalog, server_account_subject: CurrentAccountSubject) -> Self {
        Self {
            catalog,
            server_account_subject,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait(?Send)]
impl super::Command for DebugSemsearchReindexCommand {
    async fn run(&mut self) -> Result<(), InternalError> {
        // TODO: Extract auth and TX handling outside of commands
        let catalog_with_auth = self
            .catalog
            .builder_chained()
            .add_value(self.server_account_subject.clone())
            .build();

        let txr = database_common::DatabaseTransactionRunner::new(catalog_with_auth);

        txr.transactional(|catalog| async move {
            let indexer = catalog
                .get_one::<kamu_search_services::SearchServiceLocalIndexer>()
                .int_err()?;

            indexer.run_initialization().await?;

            Ok(())
        })
        .await?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
