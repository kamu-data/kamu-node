// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use init_on_startup::InitOnStartup as _;
use internal_error::*;
use kamu_search_services::NaturalLanguageSearchIndexer;

use super::{Command, CommandDesc};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[dill::component]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc {
    needs_admin_auth: true,
    needs_transaction: true,
})]
pub struct DebugSemsearchReindexCommand {
    indexer: Option<Arc<NaturalLanguageSearchIndexer>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl Command for DebugSemsearchReindexCommand {
    async fn run(&self) -> Result<(), InternalError> {
        let Some(indexer) = &self.indexer else {
            return Err(InternalError::new("Semantic search is not configured"));
        };

        indexer.run_initialization().await?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
