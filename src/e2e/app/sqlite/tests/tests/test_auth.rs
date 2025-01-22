// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu_node_e2e_common::prelude::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_login_enabled_methods
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_login_password_predefined_successful,
    options = Options::default().with_kamu_config(indoc::indoc!(
        r#"
        auth:
          providers:
              - kind: password
                accounts:
                  - accountName: kamu
                    email: kamu@example.com
        "#
    ))
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_kamu_access_token_middleware,
    options = Options::default().with_kamu_config(indoc::indoc!(
        r#"
        auth:
          providers:
              - kind: password
                accounts:
                  - accountName: kamu
                    email: kamu@example.com
        "#
    ))
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_token_validate,
    options = Options::default().with_kamu_config(indoc::indoc!(
        r#"
        auth:
          providers:
              - kind: password
                accounts:
                  - accountName: kamu
                    email: kamu@example.com
        "#
    ))
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_login_via_rest_password,
    options = Options::default().with_kamu_config(indoc::indoc!(
        r#"
        auth:
          providers:
              - kind: password
                accounts:
                  - accountName: kamu
                    email: kamu@example.com
        "#
    ))
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_login_via_rest_unauthorized,
    options = Options::default().with_kamu_config(indoc::indoc!(
        r#"
        auth:
          providers:
              - kind: password
                accounts:
                  - accountName: kamu
                    email: kamu@example.com
        "#
    ))
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
