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
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_data_room_quota_exceeded,
    extra_test_groups = "containerized, quota, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_announcements_quota_exceeded,
    extra_test_groups = "containerized, quota, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_activity_change_by_for_remove,
    extra_test_groups = "containerized, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_data_room_operations,
    extra_test_groups = "containerized, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_announcements_operations,
    extra_test_groups = "containerized, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_activity,
    extra_test_groups = "containerized, molecule",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

kamu_node_run_api_server_e2e_test!(
    storage = sqlite,
    fixture = kamu_node_e2e_repo_tests::test_molecule_v2_search,
    extra_test_groups = "containerized, molecule, search",
);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
