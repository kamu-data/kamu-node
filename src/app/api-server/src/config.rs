// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::collections::BTreeMap;
use std::path::PathBuf;

use container_runtime::{ContainerRuntimeType, NetworkNamespaceType};
use duration_string::DurationString;
use internal_error::*;
use kamu::{
    EngineConfigDatafusionEmbeddedBatchQuery,
    EngineConfigDatafusionEmbeddedCompaction,
    EngineConfigDatafusionEmbeddedIngest,
};
use kamu_accounts::{AccountConfig, DidSecretEncryptionConfig};
use kamu_datasets::DatasetEnvVarsConfig;
use odf::dataset::IpfsGateway;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub const ACCOUNT_KAMU: &str = "kamu";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
    /// Configuration for flow system
    pub flow_system: FlowSystemConfig,
    /// Ingestions sources
    pub source: SourceConfig,
    /// Outbox configuration
    pub outbox: OutboxConfig,
    /// Email gateway configuration
    pub email: EmailConfig,
    /// UNSTABLE: Identity configuration
    pub identity: Option<IdentityConfig>,
    /// Seach configuration
    pub search: Option<SearchConfig>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Runtime
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub thread_stack_size: Option<usize>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Engine
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

    /// Embedded Datafusion engine configuration
    pub datafusion_embedded: EngineConfigDatafution,
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
            datafusion_embedded: EngineConfigDatafution::default(),
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfigDatafution {
    /// Base configuration options
    /// See: https://datafusion.apache.org/user-guide/configs.html
    pub base: BTreeMap<String, String>,

    /// Ingest-specific overrides to the base config
    pub ingest: BTreeMap<String, String>,

    /// Batch query-specific overrides to the base config
    pub batch_query: BTreeMap<String, String>,

    /// Compaction-specific overrides to the base config
    pub compaction: BTreeMap<String, String>,
}

impl Default for EngineConfigDatafution {
    fn default() -> Self {
        Self {
            base: kamu::EngineConfigDatafusionEmbeddedBase::DEFAULT_SETTINGS
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            ingest: kamu::EngineConfigDatafusionEmbeddedIngest::DEFAULT_OVERRIDES
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            batch_query: kamu::EngineConfigDatafusionEmbeddedBatchQuery::DEFAULT_OVERRIDES
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            compaction: kamu::EngineConfigDatafusionEmbeddedCompaction::DEFAULT_OVERRIDES
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }
}

impl EngineConfigDatafution {
    pub fn into_system(
        self,
    ) -> Result<
        (
            kamu::EngineConfigDatafusionEmbeddedIngest,
            kamu::EngineConfigDatafusionEmbeddedBatchQuery,
            kamu::EngineConfigDatafusionEmbeddedCompaction,
        ),
        InternalError,
    > {
        let from_merged_with_base_and_defaults =
            |defaults: &[(&str, &str)], overrides: BTreeMap<String, String>| {
                kamu::EngineConfigDatafusionEmbeddedBase::new_session_config(
                    self.base
                        .clone()
                        .into_iter()
                        .chain(
                            defaults
                                .iter()
                                .map(|(k, v)| ((*k).to_string(), (*v).to_string())),
                        )
                        .chain(overrides),
                )
            };

        let ingest_config = from_merged_with_base_and_defaults(
            EngineConfigDatafusionEmbeddedIngest::DEFAULT_OVERRIDES,
            self.ingest,
        )?;
        let batch_query_config = from_merged_with_base_and_defaults(
            EngineConfigDatafusionEmbeddedBatchQuery::DEFAULT_OVERRIDES,
            self.batch_query,
        )?;
        let compaction_config = from_merged_with_base_and_defaults(
            EngineConfigDatafusionEmbeddedCompaction::DEFAULT_OVERRIDES,
            self.compaction,
        )?;

        Ok((
            kamu::EngineConfigDatafusionEmbeddedIngest(ingest_config),
            kamu::EngineConfigDatafusionEmbeddedBatchQuery(batch_query_config),
            kamu::EngineConfigDatafusionEmbeddedCompaction(compaction_config),
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Auth
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub providers: Vec<AuthProviderConfig>,
    pub did_encryption: Option<DidSecretEncryptionConfig>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum AuthProviderConfig {
    Github(AuthProviderConfigGitHub),
    Password(AuthProviderConfigPassword),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AuthProviderConfigGitHub {
    pub client_id: String,
    pub client_secret: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AuthProviderConfigPassword {
    pub accounts: Vec<AccountConfig>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Repo
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// UrlConfig
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Source
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Protocol
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolConfig {
    /// IPFS configuration
    pub ipfs: IpfsConfig,

    /// FlightSQL configuration
    pub flight_sql: FlightSqlConfig,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
        let defaults = IpfsGateway::default();
        Self {
            http_gateway: defaults.url,
            pre_resolve_dnslink: defaults.pre_resolve_dnslink,
        }
    }
}

impl IpfsConfig {
    pub fn into_gateway_config(self) -> IpfsGateway {
        IpfsGateway {
            url: self.http_gateway,
            pre_resolve_dnslink: self.pre_resolve_dnslink,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct FlightSqlConfig {
    /// Whether clients can authenticate as 'anonymous' user
    pub allow_anonymous: bool,

    /// Time after which FlightSQL client session will be forgotten and client
    /// will have to re-authroize (for authenticated clients)
    pub authed_session_expiration_timeout: DurationString,

    /// Time after which FlightSQL session context will be released to free the
    /// resources (for authenticated clients)
    pub authed_session_inactivity_timeout: DurationString,

    /// Time after which FlightSQL client session will be forgotten and client
    /// will have to re-authroize (for anonymous clients)
    pub anon_session_expiration_timeout: DurationString,

    /// Time after which FlightSQL session context will be released to free the
    /// resources (for anonymous clients)
    pub anon_session_inactivity_timeout: DurationString,
}

impl Default for FlightSqlConfig {
    fn default() -> Self {
        Self {
            allow_anonymous: true,
            authed_session_expiration_timeout: DurationString::from_string("30m".to_owned())
                .unwrap(),
            authed_session_inactivity_timeout: DurationString::from_string("5s".to_owned())
                .unwrap(),
            anon_session_expiration_timeout: DurationString::from_string("5m".to_owned()).unwrap(),
            anon_session_inactivity_timeout: DurationString::from_string("5s".to_owned()).unwrap(),
        }
    }
}

impl FlightSqlConfig {
    pub fn to_session_auth_config(&self) -> kamu_adapter_flight_sql::SessionAuthConfig {
        kamu_adapter_flight_sql::SessionAuthConfig {
            allow_anonymous: self.allow_anonymous,
        }
    }
    pub fn to_session_caching_config(&self) -> kamu_adapter_flight_sql::SessionCachingConfig {
        kamu_adapter_flight_sql::SessionCachingConfig {
            authed_session_expiration_timeout: self.authed_session_expiration_timeout.into(),
            authed_session_inactivity_timeout: self.authed_session_inactivity_timeout.into(),
            anon_session_expiration_timeout: self.anon_session_expiration_timeout.into(),
            anon_session_inactivity_timeout: self.anon_session_inactivity_timeout.into(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Database
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SqliteDatabaseConfig {
    pub database_path: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDatabaseConfig {
    pub credentials_policy: DatabaseCredentialsPolicyConfig,
    pub database_name: String,
    pub host: String,
    pub port: Option<u16>,
    pub max_connections: Option<u32>,
    pub max_lifetime_secs: Option<u64>,
    pub acquire_timeout_secs: Option<u64>,
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Helpers
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct OutboxConfig {
    pub awaiting_step_secs: Option<i64>,
    pub batch_size: Option<i64>,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            awaiting_step_secs: Some(1),
            batch_size: Some(20),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Identity
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IdentityConfig {
    /// Private key used to sign API responses.
    /// Currently only `ed25519` keys are supported.
    ///
    /// To generate use:
    ///
    ///     dd if=/dev/urandom bs=1 count=32 status=none |
    ///         base64 -w0 |
    ///         tr '+/' '-_' |
    ///         tr -d '=' |
    ///         (echo -n u && cat)
    ///
    /// The command above:
    /// - reads 32 random bytes
    /// - base64-encodes them
    /// - converts default base64 encoding to base64url and removes padding
    /// - prepends a multibase prefix
    pub private_key: Option<odf::metadata::PrivateKey>,
}

impl IdentityConfig {
    pub fn new() -> Self {
        Self { private_key: None }
    }

    pub fn to_infra_cfg(&self) -> Option<kamu_adapter_http::data::query_types::IdentityConfig> {
        self.private_key
            .clone()
            .map(|private_key| kamu_adapter_http::data::query_types::IdentityConfig { private_key })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub sender_address: String,
    pub sender_name: Option<String>,
    pub gateway: EmailConfigGateway,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum EmailConfigGateway {
    Dummy,
    Postmark(EmailConfigPostmarkGateway),
}

impl Default for EmailConfigGateway {
    fn default() -> Self {
        Self::Dummy
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfigPostmarkGateway {
    pub api_key: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct FlowSystemConfig {
    pub flow_agent: Option<FlowAgentConfig>,

    pub task_agent: Option<TaskAgentConfig>,
}

impl Default for FlowSystemConfig {
    fn default() -> Self {
        Self {
            flow_agent: Some(FlowAgentConfig::default()),
            task_agent: Some(TaskAgentConfig::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FlowAgentConfig {
    pub awaiting_step_secs: Option<i64>,
    pub mandatory_throttling_period_secs: Option<i64>,
}

impl FlowAgentConfig {
    pub fn new() -> Self {
        Self {
            awaiting_step_secs: None,
            mandatory_throttling_period_secs: None,
        }
    }
}

impl Default for FlowAgentConfig {
    fn default() -> Self {
        Self {
            awaiting_step_secs: Some(1),
            mandatory_throttling_period_secs: Some(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskAgentConfig {
    pub task_checking_interval_secs: Option<i64>,
}

impl TaskAgentConfig {
    pub fn new() -> Self {
        Self {
            task_checking_interval_secs: None,
        }
    }
}

impl Default for TaskAgentConfig {
    fn default() -> Self {
        Self {
            task_checking_interval_secs: Some(1),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Search
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SearchConfig {
    /// Indexer configuration
    pub indexer: Option<SearchIndexerConfig>,

    /// Embeddings chunker configuration
    pub embeddings_chunker: Option<EmbeddingsChunkerConfig>,

    /// Embeddings encoder configuration
    pub embeddings_encoder: EmbeddingsEncoderConfig,

    /// Vector repository configuration
    pub vector_repo: VectorRepoConfig,

    /// The multiplication factor that determines how many more points will be
    /// requested from vector store to compensate for filtering out results that
    /// may be inaccessible to user.
    #[serde(default = "SearchConfig::default_overfetch_factor")]
    pub overfetch_factor: f32,

    /// The additive value that determines how many more points will be
    /// requested from vector store to compensate for filtering out results that
    /// may be inaccessible to user.
    #[serde(default = "SearchConfig::default_overfetch_amount")]
    pub overfetch_amount: usize,

    #[serde(default = "SearchConfig::default_semantic_search_threshold_score")]
    pub semantic_search_threshold_score: f32,
}

impl SearchConfig {
    pub const DEFAULT_MODEL: &str = "text-embedding-ada-002";
    pub const DEFAULT_DIMENSIONS: usize = 1536;

    pub fn default_overfetch_factor() -> f32 {
        2.0
    }

    pub fn default_overfetch_amount() -> usize {
        10
    }

    pub fn default_semantic_search_threshold_score() -> f32 {
        0.0
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexerConfig {
    // Whether to clear and re-index on start or use existing vectors if any
    #[serde(default)]
    pub clear_on_start: bool,

    /// Whether to skip indexing datasets that have no readme or description
    #[serde(default)]
    pub skip_datasets_with_no_description: bool,

    /// Whether to skip indexing datasets that have no data
    #[serde(default)]
    pub skip_datasets_with_no_data: bool,

    /// Whether to include the original text as payload of the vectors when
    /// storing them. It is not needed for normal service operations but can
    /// help debug issues.
    #[serde(default)]
    pub payload_include_content: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum EmbeddingsChunkerConfig {
    Simple(EmbeddingsChunkerConfigSimple),
}

impl Default for EmbeddingsChunkerConfig {
    fn default() -> Self {
        Self::Simple(EmbeddingsChunkerConfigSimple::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingsChunkerConfigSimple {
    // Whether to chunk separately major dataset sections like name, schema, readme, or to combine
    // them all into one chunk
    pub split_sections: Option<bool>,

    // Whether to split section content by paragraph
    pub split_paragraphs: Option<bool>,
}

impl Default for EmbeddingsChunkerConfigSimple {
    fn default() -> Self {
        Self {
            split_sections: Some(false),
            split_paragraphs: Some(false),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum EmbeddingsEncoderConfig {
    OpenAi(EmbeddingsEncoderConfigOpenAi),
}

impl Default for EmbeddingsEncoderConfig {
    fn default() -> Self {
        Self::OpenAi(EmbeddingsEncoderConfigOpenAi::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingsEncoderConfigOpenAi {
    pub url: Option<String>,
    pub api_key: String,
    pub model_name: Option<String>,
    pub dimensions: Option<usize>,
}

impl Default for EmbeddingsEncoderConfigOpenAi {
    fn default() -> Self {
        Self {
            url: None,
            api_key: String::new(),
            model_name: Some(SearchConfig::DEFAULT_MODEL.to_string()),
            dimensions: Some(SearchConfig::DEFAULT_DIMENSIONS),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum VectorRepoConfig {
    Qdrant(VectorRepoConfigQdrant),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct VectorRepoConfigQdrant {
    pub url: String,
    pub api_key: Option<String>,
    pub collection_name: Option<String>,
    pub dimensions: Option<usize>,
}

impl Default for VectorRepoConfigQdrant {
    fn default() -> Self {
        Self {
            url: String::new(),
            api_key: None,
            collection_name: Some("kamu-datasets".to_string()),
            dimensions: Some(SearchConfig::DEFAULT_DIMENSIONS),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct VectorRepoConfigQdrantContainer {
    pub image: Option<String>,
    pub dimensions: Option<usize>,
    pub start_timeout: Option<DurationString>,
}

impl Default for VectorRepoConfigQdrantContainer {
    fn default() -> Self {
        Self {
            image: Some(kamu::utils::docker_images::QDRANT.to_string()),
            dimensions: Some(SearchConfig::DEFAULT_DIMENSIONS),
            start_timeout: Some(DurationString::from_string("30s".to_owned()).unwrap()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
