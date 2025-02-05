// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::assert_matches::assert_matches;

use kamu_adapter_http::general::NodeInfoResponse;
use kamu_node_e2e_common::{KamuApiServerClient, KamuApiServerClientExt};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_odf_info_mt(kamu_api_server_client: KamuApiServerClient) {
    assert_matches!(
        kamu_api_server_client.odf_core().info().await,
        Ok(NodeInfoResponse { is_multi_tenant })
            if is_multi_tenant
    );
}
