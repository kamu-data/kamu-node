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

kamu_node_run_api_server_e2e_test_matrix!(
    storage = postgres,
    fixture = kamu_cli_e2e_repo_tests::rest_api::test_odf_info_mt,
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
