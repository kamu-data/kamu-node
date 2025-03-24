// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use internal_error::*;

use super::{Command, CommandDesc};

#[dill::component]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc {
    needs_admin_auth: false,
    needs_transaction: false,
})]
pub struct ListMetricsCommand {
    #[dill::component(explicit)]
    metrics_registry: Arc<prometheus::Registry>,
}

#[async_trait::async_trait]
impl Command for ListMetricsCommand {
    async fn run(&self) -> Result<(), InternalError> {
        // TODO: Proper implementation is blocked by https://github.com/tikv/rust-prometheus/issues/526
        let metric_families = self.metrics_registry.gather();
        println!("{metric_families:#?}");
        Ok(())
    }
}
