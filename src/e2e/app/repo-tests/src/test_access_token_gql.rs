// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu_accounts::DEFAULT_ACCOUNT_NAME_STR;
use kamu_adapter_graphql::traits::ResponseExt;
use kamu_cli_e2e_common::{KamuApiServerClient, KamuApiServerClientExt};
use pretty_assertions::assert_eq;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_access_token_gql(mut kamu_api_server_client: KamuApiServerClient) {
    kamu_api_server_client.auth().login_as_kamu().await;

    let account_id = {
        let res = kamu_api_server_client
            .graphql_api_call_ex(
                async_graphql::Request::new(indoc::indoc!(
                    r#"
                    query ($accountName: AccountName!) {
                      accounts {
                        byName(name: $accountName) {
                          id
                        }
                      }
                    }
                    "#
                ))
                .variables(async_graphql::Variables::from_value(
                    async_graphql::value!({
                        "accountName": DEFAULT_ACCOUNT_NAME_STR,
                    }),
                )),
            )
            .await;
        assert!(res.is_ok(), "{res:?}");
        res.into_json_data()["accounts"]["byName"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // Create a new access token
    {
        let res = kamu_api_server_client
            .graphql_api_call_ex(
                async_graphql::Request::new(indoc::indoc!(
                    r#"
                    mutation ($accountId: AccountID!) {
                      accounts {
                        byId(accountId: $accountId) {
                          accessTokens {
                            createAccessToken(tokenName: "foo") {
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
                      }
                    }
                    "#
                ))
                .variables(async_graphql::Variables::from_value(
                    async_graphql::value!({
                        "accountId": account_id,
                    }),
                )),
            )
            .await;
        assert!(res.is_ok(), "{res:?}");
        assert_eq!(
            async_graphql::value!({
                "accounts": {
                    "byId": {
                        "accessTokens": {
                            "createAccessToken": {
                                "__typename": "CreateAccessTokenResultSuccess",
                                "message": "Success",
                                "token": {
                                    "name": "foo"
                                }
                            }
                        }
                    }
                }
            }),
            res.data,
        );
    }

    // Get list of access tokens
    {
        let res = kamu_api_server_client
            .graphql_api_call_ex(
                async_graphql::Request::new(indoc::indoc!(
                    r#"
                    query ($accountId: AccountID!) {
                      accounts {
                        byId(accountId: $accountId) {
                          accessTokens {
                            listAccessTokens(perPage: 10, page: 0) {
                              nodes {
                                name
                                revokedAt
                              }
                            }
                          }
                        }
                      }
                    }
                    "#
                ))
                .variables(async_graphql::Variables::from_value(
                    async_graphql::value!({
                        "accountId": account_id,
                    }),
                )),
            )
            .await;
        assert!(res.is_ok(), "{res:?}");
        assert_eq!(
            async_graphql::value!({
                "accounts": {
                    "byId": {
                        "accessTokens": {
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
                }
            }),
            res.data,
        );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
