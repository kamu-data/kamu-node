// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_group::group(resourcegen)]
#[test]
fn update_config_schema() {
    let mut resources_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    resources_path.push("../../../resources");

    let schema = setty::Config::<kamu_api_server::config::ApiServerConfig>::new()
        .json_schema()
        .to_string_pretty();
    std::fs::write(resources_path.join("api-server/config-schema.json"), schema).unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_group::group(resourcegen)]
#[test]
fn update_config_readme() {
    let mut resources_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    resources_path.push("../../../resources");

    let md = setty::Config::<kamu_api_server::config::ApiServerConfig>::new().markdown();
    std::fs::write(resources_path.join("api-server/config.md"), md).unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
