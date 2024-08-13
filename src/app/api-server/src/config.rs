// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

use container_runtime::{ContainerRuntimeType, NetworkNamespaceType};
use duration_string::DurationString;
use kamu_accounts::AccountConfig;
use kamu_datasets::DatasetEnvVarsConfig;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

pub const ACCOUNT_KAMU: &str = "kamu";

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ApiServerConfig {
    /// Authentication & authorization
    pub auth: AuthConfig,
    /// Database
    pub database: DatabaseConfig,
    /// Dataset environment variable feature
    pub dataset_env_vars: DatasetEnvVarsConfig,
    /// Ingest and transform engines
    pub engine: EngineConfig,
    /// Protocols
    pub protocol: ProtocolConfig,
    /// Tokio runtime
    pub runtime: RuntimeConfig,
    /// Dataset repository
    pub repo: RepoConfig,
    /// File upload repository
    pub upload_repo: UploadRepoConfig,
    /// External URLs
    pub url: UrlConfig,
    /// Ingestions sources
    pub source: SourceConfig,
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
// Engine
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    /// Maximum number of engine operations that can be performed concurrently
    pub max_concurrency: Option<u32>,
    /// Type of the runtime to use when running the data processing engines
    pub runtime: ContainerRuntimeType,
    /// Type of the networking namespace (relevant when running in container
    /// environments)
    pub network_ns: NetworkNamespaceType,
    /// Timeout for starting an engine container
    pub start_timeout: DurationString,
    /// Timeout for waiting the engine container to stop gracefully
    pub shutdown_timeout: DurationString,
    /// UNSTABLE: Default engine images
    pub images: EngineImagesConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let defaults = kamu::EngineProvisionerLocalConfig::default();

        Self {
            max_concurrency: Some(1),
            runtime: ContainerRuntimeType::Podman,
            network_ns: NetworkNamespaceType::Private,
            start_timeout: defaults.start_timeout.into(),
            shutdown_timeout: defaults.shutdown_timeout.into(),
            images: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EngineImagesConfig {
    /// UNSTABLE: Spark engine image
    pub spark: String,
    /// UNSTABLE: Flink engine image
    pub flink: String,
    /// UNSTABLE: Datafusion engine image
    pub datafusion: String,
    /// UNSTABLE: RisingWave engine image
    pub risingwave: String,
}

impl Default for EngineImagesConfig {
    fn default() -> Self {
        let defaults = kamu::EngineProvisionerLocalConfig::default();

        Self {
            spark: defaults.spark_image,
            flink: defaults.flink_image,
            datafusion: defaults.datafusion_image,
            risingwave: defaults.risingwave_image,
        }
    }
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
// Source
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SourceConfig {
    /// Target number of records after which we will stop consuming from the
    /// resumable source and commit data, leaving the rest for the next
    /// iteration. This ensures that one data slice doesn't become too big.
    pub target_records_per_slice: u64,
    /// MQTT-specific configuration
    pub mqtt: MqttSourceConfig,
    /// Ethereum-specific configuration
    pub ethereum: EthereumSourceConfig,
}

impl Default for SourceConfig {
    fn default() -> Self {
        let infra_cfg = kamu::ingest::SourceConfig::default();
        Self {
            target_records_per_slice: infra_cfg.target_records_per_slice,
            mqtt: MqttSourceConfig::default(),
            ethereum: EthereumSourceConfig::default(),
        }
    }
}

impl SourceConfig {
    pub fn to_infra_cfg(&self) -> kamu::ingest::SourceConfig {
        kamu::ingest::SourceConfig {
            target_records_per_slice: self.target_records_per_slice,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MqttSourceConfig {
    /// Time in milliseconds to wait for MQTT broker to send us some data after
    /// which we will consider that we have "caught up" and end the polling
    /// loop.
    pub broker_idle_timeout_ms: u64,
}

impl Default for MqttSourceConfig {
    fn default() -> Self {
        let infra_cfg = kamu::ingest::MqttSourceConfig::default();
        Self {
            broker_idle_timeout_ms: infra_cfg.broker_idle_timeout_ms,
        }
    }
}

impl MqttSourceConfig {
    pub fn to_infra_cfg(&self) -> kamu::ingest::MqttSourceConfig {
        kamu::ingest::MqttSourceConfig {
            broker_idle_timeout_ms: self.broker_idle_timeout_ms,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EthereumSourceConfig {
    /// Default RPC endpoints to use if source does not specify one explicitly.
    pub rpc_endpoints: Vec<EthRpcEndpoint>,
    /// Default number of blocks to scan within one query to `eth_getLogs` RPC
    /// endpoint.
    pub get_logs_block_stride: u64,
    /// Forces iteration to stop after the specified number of blocks were
    /// scanned even if we didn't reach the target record number. This is useful
    /// to not lose a lot of scanning progress in case of an RPC error.
    pub commit_after_blocks_scanned: u64,
}

impl Default for EthereumSourceConfig {
    fn default() -> Self {
        let infra_cfg = kamu::ingest::EthereumSourceConfig::default();
        Self {
            rpc_endpoints: Vec::new(),
            get_logs_block_stride: infra_cfg.get_logs_block_stride,
            commit_after_blocks_scanned: infra_cfg.commit_after_blocks_scanned,
        }
    }
}

impl EthereumSourceConfig {
    pub fn to_infra_cfg(&self) -> kamu::ingest::EthereumSourceConfig {
        kamu::ingest::EthereumSourceConfig {
            rpc_endpoints: self
                .rpc_endpoints
                .iter()
                .map(EthRpcEndpoint::to_infra_cfg)
                .collect(),
            get_logs_block_stride: self.get_logs_block_stride,
            commit_after_blocks_scanned: self.commit_after_blocks_scanned,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EthRpcEndpoint {
    pub chain_id: u64,
    pub chain_name: String,
    pub node_url: Url,
}

impl EthRpcEndpoint {
    pub fn to_infra_cfg(&self) -> kamu::ingest::EthRpcEndpoint {
        kamu::ingest::EthRpcEndpoint {
            chain_id: self.chain_id,
            chain_name: self.chain_name.clone(),
            node_url: self.node_url.clone(),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Protocol
/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolConfig {
    /// IPFS configuration
    pub ipfs: IpfsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IpfsConfig {
    /// HTTP Gateway URL to use for downloads.
    /// For safety, it defaults to `http://localhost:8080` - a local IPFS daemon.
    /// If you don't have IPFS installed, you can set this URL to
    /// one of the public gateways like `https://ipfs.io`.
    /// List of public gateways can be found here: `https://ipfs.github.io/public-gateway-checker/`
    pub http_gateway: Url,

    /// Whether kamu should pre-resolve IPNS DNSLink names using DNS or leave it
    /// to the Gateway.
    pub pre_resolve_dnslink: bool,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        let defaults = kamu::IpfsGateway::default();
        Self {
            http_gateway: defaults.url,
            pre_resolve_dnslink: defaults.pre_resolve_dnslink,
        }
    }
}

impl IpfsConfig {
    pub fn into_gateway_config(self) -> kamu::IpfsGateway {
        kamu::IpfsGateway {
            url: self.http_gateway,
            pre_resolve_dnslink: self.pre_resolve_dnslink,
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
    pub port: Option<u16>,
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
