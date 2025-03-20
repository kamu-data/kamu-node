// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct GqlSchemaCommand {}

impl GqlSchemaCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait(?Send)]
impl super::Command for GqlSchemaCommand {
    async fn run(&mut self) -> Result<(), InternalError> {
        println!("{}", kamu_adapter_graphql::schema().sdl());
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
