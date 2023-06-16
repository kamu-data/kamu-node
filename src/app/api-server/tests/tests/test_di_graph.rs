// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use dill::*;

#[test_log::test(tokio::test)]
async fn test_di_graph_validates() {
    let mut catalog_builder =
        kamu_api_server::init_dependencies(kamu_api_server::RunMode::RemoteS3Url);

    // TODO: We should ensure this test covers parameters requested by commands and
    // types needed for GQL/HTTP adapter that are currently being constructed
    // manually
    let validate_result = catalog_builder
        .validate()
        .ignore::<kamu::WorkspaceLayout>()
        .ignore::<dyn kamu::domain::DatasetRepository>();

    assert!(
        validate_result.is_ok(),
        "{}",
        validate_result.err().unwrap()
    );
}
