// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

use kamu_accounts::AccountConfig;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

pub const ACCOUNT_KAMU: &str = "kamu";

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ApiServerConfig {
    pub runtime: RuntimeConfig,
    pub auth: AuthConfig,
    pub repo: RepoConfig,
    pub url: UrlConfig,
    pub upload_repo: UploadRepoConfig,
    pub database: DatabaseConfig,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Runtime
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub providers: Vec<AuthProviderConfig>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum AuthProviderConfig {
    Github(AuthProviderConfigGitHub),
    Password(AuthProviderConfigPassword),
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AuthProviderConfigGitHub {
    pub client_id: String,
    pub client_secret: String,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthProviderConfigPassword {
    pub accounts: Vec<AccountConfig>,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Repo
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RepoConfig {
    #[serde(deserialize_with = "parse_repo_url_opt", default)]
    pub repo_url: Option<Url>,
    pub caching: RepoCachingConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RepoCachingConfig {
    // Caches dataset handles to avoid expensive S3 bucket scans
    pub registry_cache_enabled: bool,

    // Stores metadata blocks in a local directory to avoid many tiny S3 requests
    pub metadata_local_fs_cache_path: Option<PathBuf>,
}

/////////////////////////////////////////////////////////////////////////////////////////
// UrlConfig
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UrlConfig {
    #[serde(deserialize_with = "parse_repo_url")]
    pub base_url_platform: Url,
    #[serde(deserialize_with = "parse_repo_url")]
    pub base_url_rest: Url,
    #[serde(deserialize_with = "parse_repo_url")]
    pub base_url_flightsql: Url,
}

impl Default for UrlConfig {
    fn default() -> Self {
        Self {
            base_url_platform: Url::parse("http://localhost:4200").unwrap(),
            base_url_rest: Url::parse("http://localhost:8080").unwrap(),
            base_url_flightsql: Url::parse("grpc://localhost:50050").unwrap(),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Database
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "provider")]
pub enum DatabaseConfig {
    InMemory,
    Sqlite(SqliteDatabaseConfig),
    Postgres(RemoteDatabaseConfig),
    // MySql(RemoteDatabaseConfig),
    // MariaDB(RemoteDatabaseConfig),
}

impl DatabaseConfig {
    pub fn needs_database(&self) -> bool {
        !matches!(self, DatabaseConfig::InMemory)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::InMemory
    }
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SqliteDatabaseConfig {
    pub database_path: String,
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDatabaseConfig {
    pub credentials_policy: DatabaseCredentialsPolicyConfig,
    pub database_name: String,
    pub host: String,
    pub port: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseCredentialsPolicyConfig {
    pub source: DatabaseCredentialSourceConfig,
    pub rotation_frequency_in_minutes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum DatabaseCredentialSourceConfig {
    RawPassword(RawDatabasePasswordPolicyConfig),
    AwsSecret(AwsSecretDatabasePasswordPolicyConfig),
    AwsIamToken(AwsIamTokenPasswordPolicyConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RawDatabasePasswordPolicyConfig {
    pub user_name: String,
    pub raw_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AwsSecretDatabasePasswordPolicyConfig {
    pub secret_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AwsIamTokenPasswordPolicyConfig {
    pub user_name: String,
}

/////////////////////////////////////////////////////////////////////////////////////////
// Helpers
/////////////////////////////////////////////////////////////////////////////////////////

fn value_parse_repo_url(s: &str) -> Result<Url, &'static str> {
    match Url::parse(s) {
        Ok(url) => Ok(url),
        Err(_) => match PathBuf::from(s).canonicalize() {
            Ok(path) => Ok(Url::from_directory_path(path).unwrap()),
            Err(_) => Err(
                "Invalid repo-url, should be a path or a URL in form: file:///home/me/workspace",
            ),
        },
    }
}

fn parse_repo_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value = String::deserialize(deserializer)?;
    let url = value_parse_repo_url(value.as_str()).map_err(Error::custom)?;

    Ok(url)
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
// Upload repo

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UploadRepoConfig {
    pub max_file_size_mb: usize,
    pub storage: UploadRepoStorageConfig,
}

impl Default for UploadRepoConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 50,
            storage: UploadRepoStorageConfig::Local,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum UploadRepoStorageConfig {
    S3(UploadRepoStorageConfigS3),
    Local,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UploadRepoStorageConfigS3 {
    pub bucket_s3_url: String,
}

/////////////////////////////////////////////////////////////////////////////////////////
