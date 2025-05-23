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

kamu_node_run_flight_sql_server_e2e_test!(
    storage = postgres,
    fixture = kamu_node_e2e_repo_tests::test_flight_sql_anonymous,
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_flight_sql_server_e2e_test!(
    storage = postgres,
    fixture = kamu_node_e2e_repo_tests::test_flight_sql,
);
