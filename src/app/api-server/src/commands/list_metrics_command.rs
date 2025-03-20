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

pub struct ListMetricsCommand {
    metrics_registry: Arc<prometheus::Registry>,
}

impl ListMetricsCommand {
    pub fn new(metrics_registry: Arc<prometheus::Registry>) -> Self {
        Self { metrics_registry }
    }
}

#[async_trait::async_trait(?Send)]
impl super::Command for ListMetricsCommand {
    async fn run(&mut self) -> Result<(), InternalError> {
        // TODO: Proper implementation is blocked by https://github.com/tikv/rust-prometheus/issues/526
        let metric_families = self.metrics_registry.gather();
        println!("{metric_families:#?}");
        Ok(())
    }
}
