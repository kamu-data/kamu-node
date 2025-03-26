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
    needs_admin_auth: false,
    needs_transaction: false,
})]
pub struct GqlSchemaCommand {}

#[async_trait::async_trait]
impl Command for GqlSchemaCommand {
    async fn run(&self) -> Result<(), InternalError> {
        println!("{}", kamu_adapter_graphql::schema().sdl());
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
