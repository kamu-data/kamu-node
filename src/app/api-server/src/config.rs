// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ApiServerConfig {
    pub auth: AuthConfig,
    pub repo: RepoConfig,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Auth
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub providers: Vec<AuthProviderConfig>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum AuthProviderConfig {
    Github(AuthProviderConfigGitHub),
    Dummy(AuthProviderConfigDummy),
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AuthProviderConfigGitHub {}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthProviderConfigDummy {
    pub accounts: Vec<kamu::domain::auth::AccountInfo>,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Repo
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RepoConfig {
    pub caching: RepoCachingConfig,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RepoCachingConfig {
    // Caches dataset handles to avoid expensive S3 bucket scans
    pub registry_cache_enabled: bool,

    // Stores metadata blocks in a local directory to avoid many tiny S3 requests
    pub metadata_local_fs_cache_path: Option<PathBuf>,
}
