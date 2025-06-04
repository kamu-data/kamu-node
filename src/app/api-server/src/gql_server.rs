// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn gql_query(query: &str, full: bool, catalog: dill::Catalog) -> String {
    let gql_schema = kamu_adapter_graphql::schema();
    let response = gql_schema
        .execute(async_graphql::Request::new(query).data(catalog.clone()))
        .await;

    if full {
        serde_json::to_string_pretty(&response).unwrap()
    } else if response.is_ok() {
        serde_json::to_string_pretty(&response.data).unwrap()
    } else {
        for err in &response.errors {
            eprintln!("{err}")
        }
        // TODO: Error should be propagated as bad exit code
        "".to_owned()
    }
}
