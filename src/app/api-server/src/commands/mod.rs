// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

mod debug_depgraph_command;
mod debug_search_reindex_command;
mod debug_semsearch_reindex_command;
mod gql_query_command;
mod gql_schema_command;
mod list_metrics_command;
mod run_command;

pub use debug_depgraph_command::*;
pub use debug_search_reindex_command::*;
pub use debug_semsearch_reindex_command::*;
pub use gql_query_command::*;
pub use gql_schema_command::*;
pub use list_metrics_command::*;
pub use run_command::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    async fn run(&self) -> Result<(), internal_error::InternalError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CommandDesc {
    pub needs_admin_auth: bool,
    pub needs_transaction: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
