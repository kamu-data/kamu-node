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
use internal_error::*;
use kamu_accounts::{AccountConfig, DidSecretEncryptionConfig};
use kamu_accounts_services::PasswordPolicyConfig;
use kamu_datasets::DatasetEnvVarsConfig;
use odf::dataset::IpfsGateway;
use setty::types::UrlOrPath;
use setty::types::duration_string::DurationString;
use url::Url;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub const ACCOUNT_KAMU: &str = "kamu";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct ApiServerConfig {
    /// Authentication & authorization
    #[config(default)]
    pub auth: AuthConfig,

    /// Database
    #[config(default)]
    pub database: DatabaseConfig,

    /// Dataset environment variable feature
    #[config(default)]
    pub dataset_env_vars: DatasetEnvVarsConfig,

    /// Ingest and transform engines
    #[config(default)]
    pub engine: EngineConfig,

    /// Protocols
    #[config(default)]
    pub protocol: ProtocolConfig,

    /// Tokio runtime
    #[config(default)]
    pub runtime: RuntimeConfig,

    /// Dataset repository
    #[config(default)]
    pub repo: RepoConfig,

    /// File upload repository
    #[config(default)]
    pub upload_repo: UploadRepoConfig,

    /// External URLs
    #[config(default)]
    pub url: UrlConfig,

    /// Configuration for the flow system
    #[config(default)]
    pub flow_system: FlowSystemConfig,

    /// Configuration for webhooks
    #[config(default)]
    pub webhooks: WebhooksConfig,

    /// Ingestion's sources
    #[config(default)]
    pub source: SourceConfig,

    /// Outbox configuration
    #[config(default)]
    pub outbox: OutboxConfig,

    /// Email gateway configuration
    #[config(default = EmailConfig::dummy())]
    pub email: EmailConfig,

    /// UNSTABLE: Identity configuration
    #[config(default)]
    pub identity: Option<IdentityConfig>,

    /// Search configuration
    #[config(default)]
    pub search: SearchConfig,

    /// Default quotas configured by type
    #[config(default)]
    pub quota: QuotaConfig,

    /// Experimental and temporary module configuration
    #[config(default)]
    pub extra: ExtraConfig,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Extra
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct ExtraConfig {
    #[config(default)]
    pub graphql: kamu_adapter_graphql::GqlConfig,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Runtime
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct RuntimeConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub thread_stack_size: Option<usize>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Quota
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct QuotaConfig {
    #[config(default)]
    pub account: QuotaAccountConfig,
}

#[derive(setty::Config, setty::Default)]
pub struct QuotaAccountConfig {
    pub default_storage_limit_in_bytes: Option<u64>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Engine
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct EngineConfig {
    /// Maximum number of engine operations that can be performed concurrently
    pub max_concurrency: Option<u32>,

    /// Type of the runtime to use when running the data processing engines
    #[config(default = ContainerRuntimeType::Podman)]
    pub runtime: ContainerRuntimeType,

    /// Type of the networking namespace (relevant when running in container
    /// environments)
    #[config(default = NetworkNamespaceType::Private)]
    pub network_ns: NetworkNamespaceType,

    /// Timeout for starting an engine container
    #[config(default = kamu::EngineProvisionerLocalConfig::default().start_timeout)]
    pub start_timeout: DurationString,

    /// Timeout for waiting the engine container to stop gracefully
    #[config(default = kamu::EngineProvisionerLocalConfig::default().shutdown_timeout)]
    pub shutdown_timeout: DurationString,

    /// UNSTABLE: Default engine images
    #[config(default)]
    pub images: EngineImagesConfig,

    /// Embedded Datafusion engine configuration
    #[config(default)]
    pub datafusion_embedded: EngineConfigDatafusion,
}

#[derive(setty::Config, setty::Default)]
pub struct EngineImagesConfig {
    /// UNSTABLE: Spark engine image
    #[config(default = kamu::EngineProvisionerLocalConfig::default().spark_image)]
    pub spark: String,

    /// UNSTABLE: Flink engine image
    #[config(default = kamu::EngineProvisionerLocalConfig::default().flink_image)]
    pub flink: String,

    /// UNSTABLE: Datafusion engine image
    #[config(default = kamu::EngineProvisionerLocalConfig::default().datafusion_image)]
    pub datafusion: String,

    /// UNSTABLE: RisingWave engine image
    #[config(default = kamu::EngineProvisionerLocalConfig::default().risingwave_image)]
    pub risingwave: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct EngineConfigDatafusion {
    /// Base configuration options
    /// See: `<https://datafusion.apache.org/user-guide/configs.html>`
    #[config(default = to_map(kamu::EngineConfigDatafusionEmbeddedBase::DEFAULT_SETTINGS), combine(merge))]
    pub base: BTreeMap<String, String>,

    /// Ingest-specific overrides to the base config
    #[config(default = to_map(kamu::EngineConfigDatafusionEmbeddedIngest::DEFAULT_OVERRIDES), combine(merge))]
    pub ingest: BTreeMap<String, String>,

    /// Batch query-specific overrides to the base config
    #[config(default = to_map(kamu::EngineConfigDatafusionEmbeddedBatchQuery::DEFAULT_OVERRIDES), combine(merge))]
    pub batch_query: BTreeMap<String, String>,

    /// Compaction-specific overrides to the base config
    #[config(default = to_map(kamu::EngineConfigDatafusionEmbeddedCompaction::DEFAULT_OVERRIDES), combine(merge))]
    pub compaction: BTreeMap<String, String>,

    // TODO: Integrate this parameter better with datafusion configuration
    /// Makes arrow batches use contiguous `Binary` and `Utf8` encodings instead
    /// of more modern `BinaryView` and `Utf8View`. This is only needed for
    /// compatibility with some older libraries that don't yet support them.
    ///
    /// See: [kamu-node#277](https://github.com/kamu-data/kamu-node/issues/277)
    #[config(default = false)]
    pub use_legacy_arrow_buffer_encoding: bool,
}

fn to_map(v: &[(&str, &str)]) -> BTreeMap<String, String> {
    v.iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

impl EngineConfigDatafusion {
    // Called by `ConfigService` right after loading.
    // TODO: Consider how to incorporate this into `setty`
    pub(crate) fn merge_with_defaults(&mut self) {
        let mut default = Self::default();

        default.base.append(&mut self.base);
        default.ingest.append(&mut self.ingest);
        default.batch_query.append(&mut self.batch_query);
        default.compaction.append(&mut self.compaction);

        self.base = default.base;
        self.ingest = default.ingest;
        self.batch_query = default.batch_query;
        self.compaction = default.compaction;
    }

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
        let from_merged_with_base = |overrides: BTreeMap<String, String>| {
            kamu::EngineConfigDatafusionEmbeddedBase::new_session_config(
                self.base.clone().into_iter().chain(overrides),
            )
            .int_err()
        };

        let ingest_config = from_merged_with_base(self.ingest)?;
        let mut batch_query_config = from_merged_with_base(self.batch_query)?;
        let compaction_config = from_merged_with_base(self.compaction)?;

        batch_query_config.set_extension(std::sync::Arc::new(
            kamu::EngineConfigDatafusionEmbeddedBatchQueryExt {
                use_legacy_arrow_buffer_encoding: self.use_legacy_arrow_buffer_encoding,
            },
        ));

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

#[derive(setty::Config, setty::Default)]
pub struct AuthConfig {
    #[config(default)]
    pub jwt_secret: String,

    #[config(default)]
    pub providers: Vec<AuthProviderConfig>,

    #[config(default)]
    pub did_encryption: DidSecretEncryptionConfig,

    #[config(default)]
    pub password_policy: PasswordPolicyConfig,

    #[config(default = true)]
    pub allow_anonymous: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub enum AuthProviderConfig {
    Github(AuthProviderConfigGitHub),
    Password(AuthProviderConfigPassword),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub struct AuthProviderConfigGitHub {
    pub client_id: String,
    pub client_secret: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct AuthProviderConfigPassword {
    #[config(default)]
    pub accounts: Vec<AccountConfig>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Repo
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct RepoConfig {
    pub repo_url: Option<UrlOrPath>,

    #[config(default)]
    pub caching: RepoCachingConfig,

    pub data_blocks_page_size: Option<usize>,
}

#[derive(setty::Config, setty::Default)]
pub struct RepoCachingConfig {
    // Caches dataset handles to avoid expensive S3 bucket scans
    #[config(default)]
    pub registry_cache_enabled: bool,

    // Stores metadata blocks in a local directory to avoid many tiny S3 requests
    #[schemars(with = "Option<String>")]
    pub metadata_local_fs_cache_path: Option<PathBuf>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// UrlConfig
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct UrlConfig {
    #[config(default_str = "http://localhost:4200")]
    pub base_url_platform: UrlOrPath,

    #[config(default_str = "http://localhost:8080")]
    pub base_url_rest: UrlOrPath,

    #[config(default_str = "grpc://localhost:50050")]
    pub base_url_flightsql: UrlOrPath,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Source
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct SourceConfig {
    /// Target number of records after which we will stop consuming from the
    /// resumable source and commit data, leaving the rest for the next
    /// iteration. This ensures that one data slice doesn't become too big.
    #[config(default = kamu::ingest::SourceConfig::default().target_records_per_slice)]
    pub target_records_per_slice: u64,

    /// MQTT-specific configuration
    #[config(default)]
    pub mqtt: MqttSourceConfig,

    /// Ethereum-specific configuration
    #[config(default)]
    pub ethereum: EthereumSourceConfig,
}

impl SourceConfig {
    pub fn to_infra_cfg(&self) -> kamu::ingest::SourceConfig {
        kamu::ingest::SourceConfig {
            target_records_per_slice: self.target_records_per_slice,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct MqttSourceConfig {
    /// Time in milliseconds to wait for MQTT broker to send us some data after
    /// which we will consider that we have "caught up" and end the polling
    /// loop.
    #[config(default = kamu::ingest::MqttSourceConfig::default().broker_idle_timeout_ms)]
    pub broker_idle_timeout_ms: u64,
}

impl MqttSourceConfig {
    pub fn to_infra_cfg(&self) -> kamu::ingest::MqttSourceConfig {
        kamu::ingest::MqttSourceConfig {
            broker_idle_timeout_ms: self.broker_idle_timeout_ms,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct EthereumSourceConfig {
    /// Default RPC endpoints to use if source does not specify one explicitly.
    #[config(default, combine(merge))]
    pub rpc_endpoints: Vec<EthRpcEndpoint>,

    /// Default number of blocks to scan within one query to `eth_getLogs` RPC
    /// endpoint.
    #[config(default = kamu::ingest::EthereumSourceConfig::default().get_logs_block_stride)]
    pub get_logs_block_stride: u64,

    /// Forces iteration to stop after the specified number of blocks were
    /// scanned even if we didn't reach the target record number. This is useful
    /// to not lose a lot of scanning progress in case of an RPC error.
    #[config(default = kamu::ingest::EthereumSourceConfig::default().commit_after_blocks_scanned)]
    pub commit_after_blocks_scanned: u64,

    /// Many providers don't yet return `blockTimestamp` from `eth_getLogs` RPC
    /// endpoint and in such cases `block_timestamp` column will be `null`.
    /// If you enable this fallback the library will perform additional call to
    /// `eth_getBlock` to populate the timestam, but this may result in
    /// significant performance penalty when fetching many log records.
    ///
    /// See: [ethereum/execution-apis#295](https://github.com/ethereum/execution-apis/issues/295)
    #[config(default = kamu::ingest::EthereumSourceConfig::default().use_block_timestamp_fallback)]
    pub use_block_timestamp_fallback: bool,
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
            use_block_timestamp_fallback: self.use_block_timestamp_fallback,
        }
    }
}

#[derive(setty::Config)]
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

#[derive(setty::Config, setty::Default)]
pub struct ProtocolConfig {
    /// IPFS configuration
    #[config(default)]
    pub ipfs: IpfsConfig,

    /// FlightSQL configuration
    #[config(default)]
    pub flight_sql: FlightSqlConfig,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct IpfsConfig {
    /// HTTP Gateway URL to use for downloads.
    /// For safety, it defaults to `http://localhost:8080` - a local IPFS daemon.
    /// If you don't have IPFS installed, you can set this URL to
    /// one of the public gateways like `https://ipfs.io`.
    /// List of public gateways can be found here: `https://ipfs.github.io/public-gateway-checker/`
    #[config(default = IpfsGateway::default().url)]
    pub http_gateway: Url,

    /// Whether kamu should pre-resolve IPNS DNSLink names using DNS or leave it
    /// to the Gateway.
    #[config(default = IpfsGateway::default().pre_resolve_dnslink)]
    pub pre_resolve_dnslink: bool,
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

#[derive(setty::Config, setty::Default)]
pub struct FlightSqlConfig {
    /// Whether clients can authenticate as 'anonymous' user
    #[config(default = true)]
    pub allow_anonymous: bool,

    /// Time after which FlightSQL client session will be forgotten and client
    /// will have to re-authroize (for authenticated clients)
    #[config(default_str = "30m")]
    pub authed_session_expiration_timeout: DurationString,

    /// Time after which FlightSQL session context will be released to free the
    /// resources (for authenticated clients)
    #[config(default_str = "5s")]
    pub authed_session_inactivity_timeout: DurationString,

    /// Time after which FlightSQL client session will be forgotten and client
    /// will have to re-authroize (for anonymous clients)
    #[config(default_str = "5m")]
    pub anon_session_expiration_timeout: DurationString,

    /// Time after which FlightSQL session context will be released to free the
    /// resources (for anonymous clients)
    #[config(default_str = "5s")]
    pub anon_session_inactivity_timeout: DurationString,
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

#[derive(setty::Config, setty::Default)]
#[serde(tag = "provider")]
pub enum DatabaseConfig {
    #[default]
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub struct SqliteDatabaseConfig {
    pub database_path: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub struct RemoteDatabaseConfig {
    pub credentials_policy: DatabaseCredentialsPolicyConfig,
    pub database_name: String,
    pub host: String,
    pub port: Option<u16>,
    pub max_connections: Option<u32>,
    pub max_lifetime_secs: Option<u64>,
    pub acquire_timeout_secs: Option<u64>,
}

#[derive(setty::Config)]
pub struct DatabaseCredentialsPolicyConfig {
    pub source: DatabaseCredentialSourceConfig,
    pub rotation_frequency_in_minutes: Option<u64>,
}

#[derive(setty::Config)]
pub enum DatabaseCredentialSourceConfig {
    RawPassword(RawDatabasePasswordPolicyConfig),
    AwsSecret(AwsSecretDatabasePasswordPolicyConfig),
    AwsIamToken(AwsIamTokenPasswordPolicyConfig),
}

#[derive(setty::Config)]
pub struct RawDatabasePasswordPolicyConfig {
    pub user_name: String,
    pub raw_password: String,
}

#[derive(setty::Config)]
pub struct AwsSecretDatabasePasswordPolicyConfig {
    pub secret_name: String,
}

#[derive(setty::Config)]
pub struct AwsIamTokenPasswordPolicyConfig {
    pub user_name: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Upload repo
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct UploadRepoConfig {
    #[config(default = 50)]
    pub max_file_size_mb: usize,

    #[config(default = UploadRepoStorageConfig::Local)]
    pub storage: UploadRepoStorageConfig,
}

#[derive(setty::Config)]
pub enum UploadRepoStorageConfig {
    S3(UploadRepoStorageConfigS3),
    Local,
}

#[derive(setty::Config)]
pub struct UploadRepoStorageConfigS3 {
    pub bucket_s3_url: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct OutboxConfig {
    #[config(default = 1)]
    pub awaiting_step_secs: i64,

    #[config(default = 20)]
    pub batch_size: i64,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Identity
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
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
    #[config(combine(replace))]
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

#[derive(setty::Config)]
pub struct EmailConfig {
    pub sender_address: String,
    pub sender_name: Option<String>,
    pub gateway: EmailConfigGateway,
}

impl EmailConfig {
    // Not providing `Default` to require `sender_address` when some value is
    // provided
    pub fn dummy() -> Self {
        Self {
            sender_address: String::new(),
            sender_name: None,
            gateway: EmailConfigGateway::Dummy,
        }
    }
}

#[derive(setty::Config, setty::Default)]
pub enum EmailConfigGateway {
    #[default]
    Dummy,
    Postmark(EmailConfigPostmarkGateway),
}

#[derive(setty::Config)]
pub struct EmailConfigPostmarkGateway {
    pub api_key: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct FlowSystemConfig {
    #[config(default)]
    pub flow_agent: FlowAgentConfig,

    #[config(default)]
    pub flow_system_event_agent: FlowSystemEventAgentConfig,

    #[config(default)]
    pub task_agent: TaskAgentConfig,
}

#[derive(setty::Config, setty::Default)]
pub struct FlowAgentConfig {
    #[config(default = 1)]
    pub awaiting_step_secs: i64,

    #[config(default = 60)]
    pub mandatory_throttling_period_secs: i64,

    #[config(default, combine(merge))]
    pub default_retry_policies: BTreeMap<String, RetryPolicyConfig>,
}

#[derive(setty::Config, setty::Default)]
pub struct RetryPolicyConfig {
    pub max_attempts: Option<u32>,
    pub min_delay_secs: Option<u32>,
    pub backoff_type: Option<RetryPolicyConfigBackoffType>,
}

#[derive(setty::Config)]
pub enum RetryPolicyConfigBackoffType {
    Fixed,
    Linear,
    Exponential,
    ExponentialWithJitter,
}

#[derive(setty::Config, setty::Default)]
pub struct TaskAgentConfig {
    #[config(default = 1)]
    pub task_checking_interval_secs: i64,
}

#[derive(setty::Config, setty::Default)]
pub struct FlowSystemEventAgentConfig {
    #[config(default = 100)]
    pub min_debounce_interval_ms: u32,

    #[config(default = 60000)]
    pub max_listening_timeout_ms: u32,

    #[config(default = 100)]
    pub batch_size: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Webhooks
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct WebhooksConfig {
    #[config(default = kamu_webhooks::DEFAULT_MAX_WEBHOOK_CONSECUTIVE_FAILURES)]
    pub max_consecutive_failures: u32,

    #[config(default = kamu_webhooks::DEFAULT_WEBHOOK_DELIVERY_TIMEOUT)]
    pub delivery_timeout_secs: u32,

    #[config(default = false)]
    pub secret_encryption_enabled: bool,
    /// Represents the encryption key for the webhooks secret. This field is
    /// required if `secret_encryption_enabled` is `true` or `None`.
    ///
    /// The encryption key must be a 32-character alphanumeric string, which
    /// includes both uppercase and lowercase Latin letters (A-Z, a-z) and
    /// digits (0-9).
    ///
    /// # Example
    /// let config = WebhooksConfig {
    ///     ...
    ///     secret_encryption_enabled: Some(true),
    ///     encryption_key:
    /// Some(String::from("aBcDeFgHiJkLmNoPqRsTuVwXyZ012345")) }; ```
    pub secret_encryption_key: Option<String>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Search
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct SearchConfig {
    /// Indexer configuration
    #[config(default)]
    pub indexer: SearchIndexerConfig,

    /// Embeddings chunker configuration
    #[config(default = EmbeddingsChunkerConfig::Simple(EmbeddingsChunkerConfigSimple::default()))]
    pub embeddings_chunker: EmbeddingsChunkerConfig,

    /// Embeddings encoder configuration
    #[config(default = EmbeddingsEncoderConfig::OpenAi(EmbeddingsEncoderConfigOpenAi::default()))]
    pub embeddings_encoder: EmbeddingsEncoderConfig,

    /// Search repository configuration
    #[config(default = SearchRepositoryConfig::Dummy)]
    pub repo: SearchRepositoryConfig,

    #[config(default = 0.0)]
    pub semantic_search_threshold_score: f32,
}

impl SearchConfig {
    pub const DEFAULT_MODEL: &str = "text-embedding-ada-002";
    pub const DEFAULT_DIMENSIONS: usize = 1536;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct SearchIndexerConfig {
    /// Whether incremental indexing is enabled
    #[config(default = false)]
    pub incremental_indexing: bool,

    // Whether to clear and re-index on start or use existing vectors if any
    #[config(default = false)]
    pub clear_on_start: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub enum EmbeddingsChunkerConfig {
    Simple(EmbeddingsChunkerConfigSimple),
}

#[derive(setty::Config, setty::Default)]
pub struct EmbeddingsChunkerConfigSimple {
    // Whether to chunk separately major dataset sections like name, schema, readme, or to combine
    // them all into one chunk
    #[config(default = false)]
    pub split_sections: bool,

    // Whether to split section content by paragraph
    #[config(default = false)]
    pub split_paragraphs: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub enum EmbeddingsEncoderConfig {
    OpenAi(EmbeddingsEncoderConfigOpenAi),
    Dummy,
}

#[derive(setty::Config, setty::Default)]
pub struct EmbeddingsEncoderConfigOpenAi {
    pub url: Option<String>,

    pub api_key: Option<String>,

    #[config(default = SearchConfig::DEFAULT_MODEL)]
    pub model_name: String,

    #[config(default = SearchConfig::DEFAULT_DIMENSIONS)]
    pub dimensions: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub enum SearchRepositoryConfig {
    Dummy,
    Elasticsearch(SearchRepositoryConfigElasticsearch),
}

#[derive(setty::Config, setty::Default)]
pub struct SearchRepositoryConfigElasticsearch {
    #[config(default = "http://localhost:9200")]
    pub url: String,

    pub password: Option<String>,

    #[schemars(with = "Option<String>")]
    pub ca_cert_pem_path: Option<PathBuf>,

    #[config(default = "kamu-node")]
    pub index_prefix: String,

    #[config(default = 30)]
    pub timeout_secs: u64,

    #[config(default = false)]
    pub enable_compression: bool,

    #[config(default = SearchConfig::DEFAULT_DIMENSIONS)]
    pub embedding_dimensions: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
