// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_flight::sql::SqlInfo;
use chrono::Duration;
use crypto_utils::AesGcmEncryptor;
use dill::*;
use internal_error::*;
use kamu::domain::{DidGeneratorDefault, KamuBackgroundCatalog, TenancyConfig};
use observability::metrics::MetricsProvider;
use s3_utils::{S3Context, S3Metrics};
use tracing::{error, info, warn};
use url::Url;

use crate::commands::{Command, CommandDesc};
use crate::ui_configuration::{UIConfiguration, UIFeatureFlags};
use crate::{
    AccessTokenLifecycleNotifier,
    AccountLifecycleNotifier,
    FlowProgressNotifier,
    cli,
    commands,
    config,
    configure_database_components,
    configure_in_memory_components,
    connect_database_initially,
    spawn_password_refreshing_job,
    try_build_db_connection_settings,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_RUST_LOG: &str = "debug,";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(args: cli::Cli, config: config::ApiServerConfig) -> Result<(), InternalError> {
    let repo_url = if let Some(repo_url) = config.repo.repo_url.as_ref().cloned() {
        repo_url
    } else {
        let workspace_dir = find_workspace();
        if !workspace_dir.exists() {
            panic!(
                "Directory is not a kamu workspace: {}",
                workspace_dir.display()
            );
        }
        Url::from_directory_path(workspace_dir.join("datasets"))
            .unwrap()
            .into()
    };

    let tenancy_config = if args.multi_tenant || repo_url.scheme() != "file" {
        TenancyConfig::MultiTenant
    } else {
        TenancyConfig::SingleTenant
    };

    let local_dir = tempfile::tempdir().unwrap();

    info!(
        version = VERSION,
        args = ?std::env::args().collect::<Vec<_>>(),
        repo_url = %repo_url,
        local_dir = %local_dir.path().display(),
        config = ?config,
        "Initializing Kamu API Server",
    );

    let kamu_account_name = odf::AccountName::new_unchecked(config::ACCOUNT_KAMU);
    let server_account_subject = kamu_accounts::CurrentAccountSubject::logged(
        odf::AccountID::new_seeded_ed25519(kamu_account_name.as_bytes()),
        kamu_account_name,
    );

    let db_config = config.database.clone();

    let e2e_http_port = args
        .e2e_output_data_path
        .as_ref()
        .map(|_| container_runtime::ContainerRuntime::default().get_random_free_port())
        .transpose()
        .int_err()?;

    let catalog = init_dependencies(
        config,
        &repo_url,
        tenancy_config,
        local_dir.path(),
        e2e_http_port,
    )
    .await?
    .build();

    // Register metrics
    let metrics_registry = observability::metrics::register_all(&catalog);

    // Database requires extra actions:
    let catalog = if db_config.needs_database() {
        // Connect database and obtain a connection pool
        let catalog_with_pool = connect_database_initially(&catalog).await?;

        // Periodically refresh password in the connection pool, if configured
        spawn_password_refreshing_job(&db_config, &catalog_with_pool).await;

        catalog_with_pool
    } else {
        catalog
    };

    let catalog = CatalogBuilder::new_chained(&catalog)
        .add_value(KamuBackgroundCatalog::new(
            catalog.clone(),
            server_account_subject.clone(),
        ))
        .build();

    let command_builder: Box<dyn TypedBuilder<dyn Command>> = match args.command {
        cli::Command::Gql(c) => match c.subcommand {
            cli::Gql::Schema(_) => Box::new(commands::GqlSchemaCommand::builder().cast()),
            cli::Gql::Query(sc) => {
                Box::new(commands::GqlQueryCommand::builder(sc.query, sc.full).cast())
            }
        },
        cli::Command::Metrics(_) => {
            Box::new(commands::ListMetricsCommand::builder(metrics_registry).cast())
        }
        cli::Command::Run(c) => Box::new(
            commands::RunCommand::builder(
                server_account_subject.clone(),
                c.address,
                c.http_port,
                c.flightsql_port,
                args.e2e_output_data_path,
                e2e_http_port,
                c.read_only,
            )
            .cast(),
        ),
        cli::Command::Debug(c) => match c.subcommand {
            cli::Debug::Depgraph(_) => Box::new(commands::DebugDepgraphCommand::builder().cast()),
            cli::Debug::SearchReindex(_) => {
                Box::new(commands::DebugSearchReindexCommand::builder().cast())
            }
        },
    };

    let command_desc = command_builder
        .metadata_get_first::<CommandDesc>()
        .copied()
        .unwrap_or_default();

    let catalog = if command_desc.needs_admin_auth {
        catalog
            .builder_chained()
            .add_value(server_account_subject)
            .build()
    } else {
        catalog
    };

    maybe_transactional(
        command_desc.needs_transaction,
        catalog,
        |catalog| async move {
            let command = command_builder.get(&catalog).int_err()?;
            command.run().await
        },
    )
    .await?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn maybe_transactional<F, RF, RT, RE>(
    transactional: bool,
    catalog: dill::Catalog,
    f: F,
) -> Result<RT, RE>
where
    F: FnOnce(dill::Catalog) -> RF,
    RF: Future<Output = Result<RT, RE>>,
    RE: From<InternalError>,
{
    if !transactional {
        f(catalog).await
    } else {
        let transaction_runner = database_common::DatabaseTransactionRunner::new(catalog);

        transaction_runner
            .transactional(|transaction_catalog| async move { f(transaction_catalog).await })
            .await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn init_observability() -> observability::init::Guard {
    let guard = observability::init::auto(
        observability::config::Config::from_env_with_prefix("KAMU_OTEL_")
            .with_service_name(BINARY_NAME)
            .with_service_version(VERSION)
            .with_default_log_levels(DEFAULT_RUST_LOG),
    );

    // Redirect panics to tracing
    observability::panic_handler::set_hook_trace_panics(false);

    guard
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn load_config(path: Option<&PathBuf>) -> Result<config::ApiServerConfig, InternalError> {
    let mut fig = setty::Config::new();

    if let Some(path) = path {
        if !path.is_file() {
            return InternalError::bail(format!("Config file '{}' not found", path.display()));
        }

        fig = fig.with_source(setty::source::File::<setty::format::Yaml>::new(path));
    };

    fig = fig.with_source(setty::source::Env::<setty::format::Yaml>::new(
        "KAMU_API_SERVER_CONFIG_",
        "__",
    ));

    let mut cfg: config::ApiServerConfig = fig.extract().int_err()?;

    // TODO: Think how to make this a `setty` feature
    cfg.engine.datafusion_embedded.merge_with_defaults();

    Ok(cfg)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_dependencies(
    config: config::ApiServerConfig,
    repo_url: &Url,
    tenancy_config: TenancyConfig,
    local_dir: &Path,
    e2e_http_port: Option<u16>,
) -> Result<CatalogBuilder, InternalError> {
    // TODO: Revisit this ugly way to get metrics
    let s3_metrics_catalog = CatalogBuilder::new()
        .add_value(S3Metrics::new())
        .bind::<dyn MetricsProvider, S3Metrics>()
        .build();
    let s3_metrics = s3_metrics_catalog.get_one::<S3Metrics>().unwrap();

    let mut b = CatalogBuilder::new_chained(&s3_metrics_catalog);

    b.add_value(observability::build_info::BuildInfo {
        app_version: env!("CARGO_PKG_VERSION"),
        build_timestamp: option_env!("VERGEN_BUILD_TIMESTAMP"),
        git_describe: option_env!("VERGEN_GIT_DESCRIBE"),
        git_sha: option_env!("VERGEN_GIT_SHA"),
        git_commit_date: option_env!("VERGEN_GIT_COMMIT_DATE"),
        git_branch: option_env!("VERGEN_GIT_BRANCH"),
        rustc_semver: option_env!("VERGEN_RUSTC_SEMVER"),
        rustc_channel: option_env!("VERGEN_RUSTC_CHANNEL"),
        rustc_host_triple: option_env!("VERGEN_RUSTC_HOST_TRIPLE"),
        rustc_commit_sha: option_env!("VERGEN_RUSTC_COMMIT_HASH"),
        cargo_target_triple: option_env!("VERGEN_CARGO_TARGET_TRIPLE"),
        cargo_features: option_env!("VERGEN_CARGO_FEATURES"),
        cargo_opt_level: option_env!("VERGEN_CARGO_OPT_LEVEL"),
    });

    // TODO: Improve output multiplexing and cache interface
    let run_info_dir = local_dir.join("run");
    let cache_dir = local_dir.join("cache");
    let remote_repos_dir = local_dir.join("repos");
    std::fs::create_dir_all(&run_info_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&remote_repos_dir).unwrap();

    b.add_value(tenancy_config);

    b.add_value(kamu::domain::RunInfoDir::new(run_info_dir));
    b.add_value(kamu::domain::CacheDir::new(cache_dir));
    b.add_value(kamu::RemoteReposDir::new(remote_repos_dir));

    b.add::<time_source::SystemTimeSourceDefault>();

    b.add::<DidGeneratorDefault>();

    b.add_value(prometheus::Registry::new_custom(Some("kamu_api_server".into()), None).unwrap());

    b.add::<container_runtime::ContainerRuntime>();
    b.add_value(container_runtime::ContainerRuntimeConfig {
        runtime: config.engine.runtime,
        network_ns: config.engine.network_ns,
    });

    // Engine config
    {
        b.add::<kamu::EngineProvisionerLocal>();
        b.add_value(kamu::EngineProvisionerLocalConfig {
            max_concurrency: config.engine.max_concurrency,
            start_timeout: config.engine.start_timeout.into(),
            shutdown_timeout: config.engine.shutdown_timeout.into(),
            spark_image: config.engine.images.spark,
            flink_image: config.engine.images.flink,
            datafusion_image: config.engine.images.datafusion,
            risingwave_image: config.engine.images.risingwave,
        });

        let (ingest_config, batch_config, compact_config) =
            config.engine.datafusion_embedded.into_system()?;

        b.add_value(ingest_config);
        b.add_value(batch_config);
        b.add_value(compact_config);
    }
    //

    b.add_value(config.protocol.ipfs.into_gateway_config());
    b.add_value(kamu::utils::ipfs_wrapper::IpfsClient::default());

    // GraphQL
    b.add_value(config.extra.graphql);
    //

    // FlightSQL
    let mut sql_info = kamu_adapter_flight_sql::sql_info::default_sql_info();
    sql_info.append(SqlInfo::FlightSqlServerName, crate::BINARY_NAME);
    sql_info.append(SqlInfo::FlightSqlServerVersion, crate::VERSION);
    b.add_value(sql_info.build().unwrap());

    b.add_value(config.protocol.flight_sql.to_session_auth_config());
    b.add_value(config.protocol.flight_sql.to_session_caching_config());

    b.add::<kamu_adapter_flight_sql::SessionAuthAnonymous>();
    b.add::<kamu_adapter_flight_sql::SessionManagerCaching>();
    b.add::<kamu_adapter_flight_sql::SessionManagerCachingState>();
    b.add::<kamu_adapter_flight_sql::KamuFlightSqlService>();
    //

    b.add::<kamu::FetchService>();
    b.add_value(config.source.to_infra_cfg());
    b.add_value(config.source.mqtt.to_infra_cfg());
    b.add_value(config.source.ethereum.to_infra_cfg());

    if let Some(identity_config) = config
        .identity
        .as_ref()
        .and_then(|identity| identity.to_infra_cfg())
    {
        b.add_value(identity_config);
    }

    b.add::<odf::dataset::DatasetFactoryImpl>();
    b.add::<kamu::ObjectStoreRegistryImpl>();
    b.add::<kamu::RemoteAliasesRegistryImpl>();
    b.add::<kamu::RemoteAliasResolverImpl>();

    kamu_adapter_flow_dataset::register_dependencies(&mut b, Default::default());
    kamu_adapter_flow_webhook::register_dependencies(&mut b);
    kamu_adapter_task_dataset::register_dependencies(&mut b);
    kamu_adapter_task_webhook::register_dependencies(&mut b);

    b.add::<kamu::RemoteRepositoryRegistryImpl>();

    b.add::<kamu::utils::simple_transfer_protocol::SimpleTransferProtocol>();
    b.add::<kamu_adapter_http::SmartTransferProtocolClientWs>();

    b.add::<kamu::DataFormatRegistryImpl>();

    b.add::<kamu::PollingIngestServiceImpl>();
    b.add::<kamu::PushIngestPlannerImpl>();
    b.add::<kamu::PushIngestExecutorImpl>();
    b.add::<kamu::TransformRequestPlannerImpl>();
    b.add::<kamu::TransformElaborationServiceImpl>();
    b.add::<kamu::TransformExecutorImpl>();
    b.add::<kamu::SyncServiceImpl>();
    b.add::<kamu::SyncRequestBuilder>();
    b.add::<kamu::CompactionPlannerImpl>();
    b.add::<kamu::CompactionExecutorImpl>();
    b.add::<kamu::VerificationServiceImpl>();
    b.add::<kamu::PullRequestPlannerImpl>();
    b.add::<kamu::PushRequestPlannerImpl>();
    b.add::<kamu::SetWatermarkPlannerImpl>();
    b.add::<kamu::SetWatermarkExecutorImpl>();
    b.add::<kamu::MetadataQueryServiceImpl>();
    b.add::<kamu::QueryServiceImpl>();
    b.add::<kamu::SchemaServiceImpl>();
    b.add::<kamu::SessionContextBuilder>();
    b.add::<kamu::ResetPlannerImpl>();
    b.add::<kamu::ResetExecutorImpl>();
    b.add::<kamu::RemoteStatusServiceImpl>();

    b.add::<kamu::CompactDatasetUseCaseImpl>();
    b.add::<kamu::PushIngestDataUseCaseImpl>();
    b.add::<kamu::PullDatasetUseCaseImpl>();
    b.add::<kamu::PushDatasetUseCaseImpl>();
    b.add::<kamu::ResetDatasetUseCaseImpl>();
    b.add::<kamu::SetWatermarkUseCaseImpl>();
    b.add::<kamu::VerifyDatasetUseCaseImpl>();
    b.add::<kamu::GetDatasetSchemaUseCaseImpl>();
    b.add::<kamu::QueryDatasetDataUseCaseImpl>();

    b.add_builder(messaging_outbox::OutboxImmediateImpl::builder(
        messaging_outbox::ConsumerFilter::ImmediateConsumers,
    ));
    b.add::<messaging_outbox::OutboxTransactionalImpl>();
    b.add::<messaging_outbox::OutboxDispatchingImpl>();
    b.bind::<dyn messaging_outbox::Outbox, messaging_outbox::OutboxDispatchingImpl>();
    b.add::<messaging_outbox::OutboxAgentImpl>();
    b.add::<messaging_outbox::OutboxAgentMetrics>();

    messaging_outbox::register_message_dispatcher::<kamu_datasets::DatasetLifecycleMessage>(
        &mut b,
        kamu_datasets::MESSAGE_PRODUCER_KAMU_DATASET_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_datasets::DatasetReferenceMessage>(
        &mut b,
        kamu_datasets::MESSAGE_PRODUCER_KAMU_DATASET_REFERENCE_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_datasets::DatasetDependenciesMessage>(
        &mut b,
        kamu_datasets::MESSAGE_PRODUCER_KAMU_DATASET_DEPENDENCY_GRAPH_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_task_system::TaskProgressMessage>(
        &mut b,
        kamu_task_system::MESSAGE_PRODUCER_KAMU_TASK_AGENT,
    );
    messaging_outbox::register_message_dispatcher::<
        kamu_flow_system::FlowConfigurationUpdatedMessage,
    >(
        &mut b,
        kamu_flow_system::MESSAGE_PRODUCER_KAMU_FLOW_CONFIGURATION_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_flow_system::FlowTriggerUpdatedMessage>(
        &mut b,
        kamu_flow_system::MESSAGE_PRODUCER_KAMU_FLOW_TRIGGER_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_flow_system::FlowProcessLifecycleMessage>(
        &mut b,
        kamu_flow_system::MESSAGE_PRODUCER_KAMU_FLOW_PROCESS_STATE_PROJECTOR,
    );
    messaging_outbox::register_message_dispatcher::<kamu_accounts::AccessTokenLifecycleMessage>(
        &mut b,
        kamu_accounts::MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_accounts::AccountLifecycleMessage>(
        &mut b,
        kamu_accounts::MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_datasets::DatasetExternallyChangedMessage>(
        &mut b,
        kamu_datasets::MESSAGE_PRODUCER_KAMU_HTTP_ADAPTER,
    );
    messaging_outbox::register_message_dispatcher::<kamu_auth_rebac::RebacDatasetPropertiesMessage>(
        &mut b,
        kamu_auth_rebac::MESSAGE_PRODUCER_KAMU_REBAC_DATASET_PROPERTIES_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_auth_rebac::RebacDatasetRelationsMessage>(
        &mut b,
        kamu_auth_rebac::MESSAGE_PRODUCER_KAMU_REBAC_DATASET_RELATIONS_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<
        kamu_webhooks::WebhookSubscriptionLifecycleMessage,
    >(
        &mut b,
        kamu_webhooks::MESSAGE_PRODUCER_KAMU_WEBHOOK_SUBSCRIPTION_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<
        kamu_webhooks::WebhookSubscriptionEventChangesMessage,
    >(
        &mut b,
        kamu_webhooks::MESSAGE_PRODUCER_KAMU_WEBHOOK_SUBSCRIPTION_EVENT_CHANGES_SERVICE,
    );

    b.add_value(messaging_outbox::OutboxConfig::new(
        Duration::seconds(config.outbox.awaiting_step_secs),
        config.outbox.batch_size,
    ));

    // Webhooks configuration
    let webhooks_config = config.webhooks;
    {
        let max_consecutive_failures = webhooks_config.max_consecutive_failures;
        assert!(
            max_consecutive_failures > 0,
            "Webhooks max_consecutive_failures must be > 0"
        );

        let delivery_timeout_secs = webhooks_config.delivery_timeout_secs;
        assert!(
            delivery_timeout_secs > 0,
            "Webhooks delivery_timeout_secs must be > 0"
        );

        match webhooks_config.secret_encryption_key.as_ref() {
            None => {
                if webhooks_config.secret_encryption_enabled {
                    panic!("Webhook secrets encryption key is required")
                } else {
                    warn!(
                        "Webhook encryption configuration is missing. Secrets will not be \
                         encrypted"
                    );
                }
            }
            Some(encryption_key) => {
                if webhooks_config.secret_encryption_enabled {
                    assert!(
                        AesGcmEncryptor::try_new(encryption_key).is_ok(),
                        "Invalid webhook secrets encryption key",
                    );
                } else {
                    warn!("Webhook encryption will be disabled");
                }
            }
        }

        b.add_value(kamu_webhooks::WebhooksConfig::new(
            max_consecutive_failures,
            Duration::seconds(i64::from(delivery_timeout_secs)),
            webhooks_config.secret_encryption_key,
        ));
    }

    b.add_value(kamu_task_system::TaskAgentConfig::new(Duration::seconds(
        config.flow_system.task_agent.task_checking_interval_secs,
    )));
    kamu_task_system_services::register_dependencies(&mut b);

    let flow_system_event_agent_config = config.flow_system.flow_system_event_agent;
    b.add_value(kamu_flow_system::FlowSystemEventAgentConfig {
        min_debounce_interval: std::time::Duration::from_millis(u64::from(
            flow_system_event_agent_config.min_debounce_interval_ms,
        )),
        max_listening_timeout: std::time::Duration::from_millis(u64::from(
            flow_system_event_agent_config.max_listening_timeout_ms,
        )),
        batch_size: flow_system_event_agent_config.batch_size,
    });

    let flow_agent_config = config.flow_system.flow_agent;
    b.add_value(kamu_flow_system::FlowAgentConfig::new(
        Duration::seconds(flow_agent_config.awaiting_step_secs),
        Duration::seconds(flow_agent_config.mandatory_throttling_period_secs),
        flow_agent_config
            .default_retry_policies
            .iter()
            .map(|(flow_type, retry_policy_config)| {
                (
                    flow_type.clone(),
                    kamu_flow_system::RetryPolicy::new(
                        retry_policy_config.max_attempts.unwrap_or(0),
                        retry_policy_config.min_delay_secs.unwrap_or(0),
                        match retry_policy_config.backoff_type {
                            Some(config::RetryPolicyConfigBackoffType::Exponential) => {
                                kamu_flow_system::RetryBackoffType::Exponential
                            }
                            Some(config::RetryPolicyConfigBackoffType::Linear) => {
                                kamu_flow_system::RetryBackoffType::Linear
                            }
                            Some(config::RetryPolicyConfigBackoffType::ExponentialWithJitter) => {
                                kamu_flow_system::RetryBackoffType::ExponentialWithJitter
                            }
                            Some(config::RetryPolicyConfigBackoffType::Fixed) | None => {
                                kamu_flow_system::RetryBackoffType::Fixed
                            }
                        },
                    ),
                )
            })
            .collect(),
    ));

    kamu_flow_system_services::register_dependencies(&mut b);

    kamu_adapter_auth_oso_rebac::register_dependencies(&mut b);
    kamu_datasets_services::register_dependencies(
        &mut b,
        kamu_datasets_services::DatasetDomainDependenciesOptions {
            needs_indexing: true,
            incremental_search_indexing: config.search.indexer.incremental_indexing,
        },
    );

    kamu_auth_rebac_services::register_dependencies(&mut b, true);
    kamu_webhooks_services::register_dependencies(&mut b);

    b.add::<odf::dataset::DummyOdfServerAccessTokenResolver>();

    configure_repository(&mut b, repo_url, &config.repo, &s3_metrics).await;

    kamu_accounts_services::register_dependencies(
        &mut b,
        kamu_accounts_services::AccountDomainDependenciesOptions {
            needs_indexing: true,
            production: true,
            incremental_search_indexing: config.search.indexer.incremental_indexing,
        },
    );

    let mut need_to_add_default_predefined_accounts_config = true;

    match tenancy_config {
        TenancyConfig::SingleTenant => {
            b.add_value(kamu_accounts::PredefinedAccountsConfig::single_tenant());
            need_to_add_default_predefined_accounts_config = false;
        }
        TenancyConfig::MultiTenant => {
            kamu_auth_web3_services::register_dependencies(&mut b);
            kamu_adapter_auth_web3::register_dependencies(&mut b);

            for provider in config.auth.providers {
                match provider {
                    config::AuthProviderConfig::Github(github_config) => {
                        kamu_adapter_oauth::register_dependencies(&mut b, true);

                        b.add_value(kamu_adapter_oauth::GithubAuthenticationConfig::new(
                            github_config.client_id,
                            github_config.client_secret,
                        ));
                    }
                    config::AuthProviderConfig::Password(prov) => {
                        b.add_value(kamu_accounts::PredefinedAccountsConfig {
                            predefined: prov.accounts,
                        });
                        need_to_add_default_predefined_accounts_config = false
                    }
                }
            }
        }
    }

    if need_to_add_default_predefined_accounts_config {
        b.add_value(kamu_accounts::PredefinedAccountsConfig::default());
    }

    b.add_value(kamu_accounts::AuthConfig {
        allow_anonymous: config.auth.allow_anonymous,
        ..kamu_accounts::AuthConfig::default()
    });

    {
        let mut protocols = kamu::domain::Protocols {
            base_url_platform: config.url.base_url_platform.into(),
            base_url_rest: config.url.base_url_rest.into(),
            base_url_flightsql: config.url.base_url_flightsql.into(),
        };

        if let Some(e2e_http_port) = e2e_http_port {
            let base_url_rest = Url::parse(&format!("http://127.0.0.1:{e2e_http_port}"))
                .expect("URL failed to parse");

            protocols.base_url_rest = base_url_rest;
        }

        b.add_value(kamu::domain::ServerUrlConfig::new(protocols));
    }

    let maybe_jwt_secret = if !config.auth.jwt_secret.is_empty() {
        Some(config.auth.jwt_secret)
    } else {
        None
    };

    b.add_value(kamu_accounts::JwtAuthenticationConfig::new(
        maybe_jwt_secret,
    ));

    b.add_value(kamu::domain::FileUploadLimitConfig::new_in_mb(
        config.upload_repo.max_file_size_mb,
    ));

    match config.upload_repo.storage {
        config::UploadRepoStorageConfig::Local => {
            b.add::<kamu_adapter_http::platform::UploadServiceLocal>();
        }
        config::UploadRepoStorageConfig::S3(s3_config) => {
            let s3_upload_direct_url = Url::parse(&s3_config.bucket_s3_url).unwrap();
            let s3_context = S3Context::from_url(&s3_upload_direct_url)
                .await
                .with_metrics(s3_metrics);

            b.add_builder(kamu_adapter_http::platform::UploadServiceS3::builder(
                s3_context,
            ));
        }
    }

    let dataset_env_vars_config = &config.dataset_env_vars;
    match dataset_env_vars_config.encryption_key.as_ref() {
        None => {
            if dataset_env_vars_config.enabled {
                panic!("Dataset env vars encryption key is required");
            } else {
                error!("Dataset env vars configuration is missing. Feature will be disabled");
            }
            b.add::<kamu_datasets_services::DatasetKeyValueServiceSysEnv>();
            b.add::<kamu_datasets_services::DatasetEnvVarServiceNull>();
        }
        Some(encryption_key) => {
            if dataset_env_vars_config.enabled {
                warn!("Dataset env vars feature will be disabled");
                b.add::<kamu_datasets_services::DatasetKeyValueServiceSysEnv>();
                b.add::<kamu_datasets_services::DatasetEnvVarServiceNull>();
            } else {
                assert!(
                    AesGcmEncryptor::try_new(encryption_key).is_ok(),
                    "Invalid dataset env var encryption key. Key must be a 32-character \
                     alphanumeric string",
                );
                b.add::<kamu_datasets_services::DatasetKeyValueServiceImpl>();
                b.add::<kamu_datasets_services::DatasetEnvVarServiceImpl>();
            }
        }
    }
    b.add_value(config.dataset_env_vars.clone());

    // Did secret key encryption configuration
    b.add_value(config.auth.did_encryption.clone());

    let did_encryption_config = config.auth.did_encryption;
    if let Some(encryption_key) = &did_encryption_config.encryption_key
        && did_encryption_config.is_enabled()
    {
        assert!(
            AesGcmEncryptor::try_new(encryption_key).is_ok(),
            "Invalid did secret encryption key",
        );
    } else {
        warn!("Did secret keys will not be stored");
    }
    //

    b.add::<database_common::DatabaseTransactionRunner>();

    configure_email_gateway(&mut b, &config.email)?;
    b.add::<AccessTokenLifecycleNotifier>();
    b.add::<AccountLifecycleNotifier>();
    b.add::<FlowProgressNotifier>();

    let maybe_db_connection_settings = try_build_db_connection_settings(&config.database);
    if let Some(db_connection_settings) = maybe_db_connection_settings {
        configure_database_components(&mut b, &config.database, db_connection_settings);
    } else {
        configure_in_memory_components(&mut b);
    };

    // Search configuration

    b.add_value(kamu_search::SearchIndexerConfig {
        clear_on_start: config.search.indexer.clear_on_start,
    });

    b.add::<kamu_search_services::EmbeddingsProviderImpl>();

    match config.search.embeddings_chunker {
        config::EmbeddingsChunkerConfig::Simple(cfg) => {
            b.add::<kamu_search_services::EmbeddingsChunkerSimple>();
            b.add_value(kamu_search_services::EmbeddingsChunkerConfigSimple {
                split_sections: cfg.split_sections,
                split_paragraphs: cfg.split_paragraphs,
            });
        }
    }

    match config.search.embeddings_encoder {
        config::EmbeddingsEncoderConfig::OpenAi(cfg) => {
            b.add::<kamu_search_openai::EmbeddingsEncoderOpenAi>();
            b.add_value(kamu_search_openai::EmbeddingsEncoderConfigOpenAI {
                url: cfg.url,
                api_key: cfg.api_key.map(Into::into),
                model_name: cfg.model_name,
                dimensions: cfg.dimensions,
            });
        }

        config::EmbeddingsEncoderConfig::Dummy => {
            b.add::<kamu_search_services::DummyEmbeddingsEncoder>();
        }
    }

    match config.search.repo {
        config::SearchRepositoryConfig::Dummy => {
            b.add::<kamu_search_services::DummySearchService>();
        }

        config::SearchRepositoryConfig::Elasticsearch(cfg) => {
            b.add::<kamu_search_services::SearchServiceImpl>();
            b.add::<kamu_search_services::SearchIndexerImpl>();
            b.add::<kamu_search_elasticsearch::ElasticsearchRepository>();
            b.add_value(kamu_search_elasticsearch::ElasticsearchClientConfig {
                url: url::Url::parse(&cfg.url).int_err()?,
                password: cfg.password,
                timeout_secs: cfg.timeout_secs,
                enable_compression: cfg.enable_compression,
                ca_cert_pem_path: cfg.ca_cert_pem_path,
            });
            b.add_value(kamu_search_elasticsearch::ElasticsearchRepositoryConfig {
                index_prefix: cfg.index_prefix,
                embedding_dimensions: cfg.embedding_dimensions,
            });
        }
    }

    //

    // UI
    b.add_value(UIConfiguration {
        ingest_upload_file_limit_mb: config.upload_repo.max_file_size_mb,
        semantic_search_threshold_score: config.search.semantic_search_threshold_score,
        min_new_password_length: config.auth.password_policy.min_new_password_length,
        feature_flags: UIFeatureFlags {
            enable_dataset_env_vars_management: config.dataset_env_vars.is_enabled(),
            allow_anonymous: config.auth.allow_anonymous,
            ..UIFeatureFlags::default()
        },
    });
    //

    Ok(b)
}

async fn configure_repository(
    b: &mut CatalogBuilder,
    repo_url: &Url,
    config: &config::RepoConfig,
    s3_metrics: &Arc<S3Metrics>,
) {
    b.add_value(
        config
            .data_blocks_page_size
            .map(
                |data_blocks_page_size| kamu_datasets_services::MetadataChainDbBackedConfig {
                    data_blocks_page_size,
                },
            )
            .unwrap_or_default(),
    );

    match repo_url.scheme() {
        "file" => {
            use odf::dataset::DatasetStorageUnitLocalFs;

            let datasets_dir = repo_url.to_file_path().unwrap();

            b.add_builder(DatasetStorageUnitLocalFs::builder(datasets_dir.clone()));
            b.add::<kamu_datasets_services::DatasetLfsBuilderDatabaseBackedImpl>();

            b.add::<kamu::ObjectStoreBuilderLocalFs>();
        }
        "s3" | "s3+http" | "s3+https" => {
            use odf::dataset::DatasetStorageUnitS3;

            let s3_context = S3Context::from_url(repo_url)
                .await
                .with_metrics(s3_metrics.clone());

            if config.caching.registry_cache_enabled {
                b.add::<odf::dataset::S3RegistryCache>();
            }

            // TODO: consider removing this local FS cache completely,
            // as metadata is now cached in the database layer.

            let metadata_cache_local_fs_path = config
                .caching
                .metadata_local_fs_cache_path
                .clone()
                .map(Arc::new);

            b.add_builder(DatasetStorageUnitS3::builder(s3_context.clone()));
            b.add_builder(
                kamu_datasets_services::DatasetS3BuilderDatabaseBackedImpl::builder()
                    .with_metadata_cache_local_fs_path(metadata_cache_local_fs_path),
            );

            let allow_http = repo_url.scheme() == "s3+http";
            b.add_builder(kamu::ObjectStoreBuilderS3::builder(s3_context, allow_http));

            // LFS object store is still needed for Datafusion operations that create
            // temporary file, such as push ingest
            b.add::<kamu::ObjectStoreBuilderLocalFs>();
        }
        _ => panic!("Unsupported repository scheme: {}", repo_url.scheme()),
    }
}

fn configure_email_gateway(
    catalog_builder: &mut dill::CatalogBuilder,
    email_config: &config::EmailConfig,
) -> Result<(), InternalError> {
    let email_config = match &email_config.gateway {
        config::EmailConfigGateway::Dummy => email_gateway::EmailConfig::Dummy,
        config::EmailConfigGateway::Postmark(postmark_config) => {
            email_gateway::EmailConfig::Postmark(email_gateway::PostmarkGatewaySettings {
                sender_address: email_utils::Email::parse(email_config.sender_address.as_str())
                    .int_err()?,
                sender_name: email_config.sender_name.clone(),
                api_key: secrecy::SecretString::from(postmark_config.api_key.as_str()),
            })
        }
    };

    email_gateway::register_dependencies(catalog_builder, email_config);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Workspace
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn find_workspace() -> PathBuf {
    let cwd = Path::new(".").canonicalize().unwrap();
    if let Some(ws) = find_workspace_rec(&cwd) {
        ws
    } else {
        cwd.join(".kamu")
    }
}

fn find_workspace_rec(p: &Path) -> Option<PathBuf> {
    let root_dir = p.join(".kamu");
    if root_dir.exists() {
        Some(root_dir)
    } else if let Some(parent) = p.parent() {
        find_workspace_rec(parent)
    } else {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
