// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn gql_schema(catalog: dill::Catalog) -> kamu_adapter_graphql::Schema {
    kamu_adapter_graphql::schema_builder(catalog)
        .extension(kamu_adapter_graphql::extensions::Tracing)
        .extension(async_graphql::extensions::ApolloTracing)
        .finish()
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn gql_query(query: &str, full: bool, catalog: dill::Catalog) -> String {
    let gql_schema = gql_schema(catalog.clone());
    let response = gql_schema.execute(query).await;

    if full {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        if response.is_ok() {
            serde_json::to_string_pretty(&response.data).unwrap()
        } else {
            for err in &response.errors {
                eprintln!("{}", err)
            }
            // TODO: Error should be propagated as bad exit code
            "".to_owned()
        }
    }
}
