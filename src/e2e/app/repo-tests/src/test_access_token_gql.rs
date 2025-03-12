// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu_accounts::DEFAULT_ACCOUNT_ID;
use kamu_cli_e2e_common::{KamuApiServerClient, KamuApiServerClientExt};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_access_token_gql(mut kamu_api_server_client: KamuApiServerClient) {
    kamu_api_server_client.auth().login_as_kamu().await;

    // Create a new access token
    kamu_api_server_client
        .graphql_api_call_assert(
            indoc::indoc!(
                r#"
                mutation {
                    auth {
                        createAccessToken (accountId: "<account_id>", tokenName: "foo") {
                            __typename
                            message
                            ... on CreateAccessTokenResultSuccess {
                                token {
                                    name
                                }
                            }
                        }
                    }
                }
                "#,
            )
            .replace("<account_id>", DEFAULT_ACCOUNT_ID.to_string().as_str())
            .as_str(),
            Ok(indoc::indoc!(
                r#"
                {
                  "auth": {
                    "createAccessToken": {
                      "__typename": "CreateAccessTokenResultSuccess",
                      "message": "Success",
                      "token": {
                        "name": "foo"
                      }
                    }
                  }
                }
                "#,
            )),
        )
        .await;

    // Get list of access tokens
    kamu_api_server_client
        .graphql_api_call_assert(
            indoc::indoc!(
                r#"
                query {
                    auth {
                        listAccessTokens (accountId: "<account_id>", perPage: 10, page: 0) {
                            nodes {
                                name,
                                revokedAt
                            }
                        }
                    }
                }
                "#,
            )
            .replace("<account_id>", DEFAULT_ACCOUNT_ID.to_string().as_str())
            .as_str(),
            Ok(indoc::indoc!(
                r#"
                {
                  "auth": {
                    "listAccessTokens": {
                      "nodes": [
                        {
                          "name": "foo",
                          "revokedAt": null
                        }
                      ]
                    }
                  }
                }
                "#,
            )),
        )
        .await;
}
