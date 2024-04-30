// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ApiServerConfig {
    pub runtime: RuntimeConfig,
    pub auth: AuthConfig,
    pub repo: RepoConfig,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Runtime
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub thread_stack_size: Option<usize>,
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
    #[serde(deserialize_with = "parse_repo_url_opt", default)]
    pub repo_url: Option<Url>,
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

/////////////////////////////////////////////////////////////////////////////////////////

fn value_parse_repo_url(s: &str) -> Result<Url, &'static str> {
    match Url::parse(s) {
        Ok(url) => Ok(url),
        Err(_) => match PathBuf::from(s).canonicalize() {
            Ok(path) => Ok(Url::from_file_path(path).unwrap()),
            Err(_) => Err(
                "Invalid repo-url, should be a path or a URL in form: file:///home/me/workspace",
            ),
        },
    }
}

fn parse_repo_url_opt<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let maybe_value = Option::<String>::deserialize(deserializer)?;

    match maybe_value {
        Some(value) => {
            let url = value_parse_repo_url(value.as_str()).map_err(Error::custom)?;

            Ok(Some(url))
        }
        None => Ok(None),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
