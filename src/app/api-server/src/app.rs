// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::{Future, IntoFuture};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use arrow_flight::sql::SqlInfo;
use chrono::Duration;
use dill::{CatalogBuilder, Component};
use internal_error::*;
use kamu::domain::{DidGeneratorDefault, TenancyConfig};
use observability::metrics::MetricsProvider;
use s3_utils::{S3Context, S3Metrics};
use tracing::{error, info, warn};
use url::Url;

use crate::ui_configuration::{UIConfiguration, UIFeatureFlags};
use crate::{
    cli,
    config,
    configure_database_components,
    configure_in_memory_components,
    connect_database_initially,
    spawn_password_refreshing_job,
    try_build_db_connection_settings,
    AccessTokenLifecycleNotifier,
    AccountLifecycleNotifier,
    FlowProgressNotifier,
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
        Url::from_directory_path(workspace_dir.join("datasets")).unwrap()
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
        true,
    );

    let db_config = config.database.clone();

    let ui_config = UIConfiguration {
        ingest_upload_file_limit_mb: config.upload_repo.max_file_size_mb,
        feature_flags: UIFeatureFlags {
            enable_dataset_env_vars_management: config.dataset_env_vars.is_enabled(),
            ..UIFeatureFlags::default()
        },
    };

    let catalog = init_dependencies(config, &repo_url, tenancy_config, local_dir.path())
        .await?
        .build();

    // Register metrics
    let metrics_registry = observability::metrics::register_all(&catalog);

    // Database requires extra actions:
    let final_catalog = if db_config.needs_database() {
        // Connect database and obtain a connection pool
        let catalog_with_pool = connect_database_initially(&catalog).await?;

        // Periodically refresh password in the connection pool, if configured
        spawn_password_refreshing_job(&db_config, &catalog_with_pool).await;

        catalog_with_pool
    } else {
        catalog
    };

    initialize_components(&final_catalog, server_account_subject.clone()).await?;

    match args.command {
        cli::Command::Gql(c) => match c.subcommand {
            cli::Gql::Schema(_) => {
                println!("{}", kamu_adapter_graphql::schema().sdl());
                Ok(())
            }
            cli::Gql::Query(sc) => {
                let result = crate::gql_server::gql_query(&sc.query, sc.full, final_catalog).await;
                print!("{}", result);
                Ok(())
            }
        },
        cli::Command::Metrics(_) => {
            // TODO: Proper implementation is blocked by https://github.com/tikv/rust-prometheus/issues/526
            let metric_families = metrics_registry.gather();
            println!("{metric_families:#?}");
            Ok(())
        }
        cli::Command::Run(c) => {
            let shutdown_requested = graceful_shutdown::trap_signals();

            let address = c
                .address
                .unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1).into());

            // API servers are built from the regular catalog
            // that does not contain any auth subject, thus they will rely on
            // their own middlewares to authenticate per request / session and execute
            // all processing in the user context.
            let (http_server, local_addr, maybe_shutdown_notify) =
                crate::http_server::build_server(
                    address,
                    c.http_port,
                    final_catalog.clone(),
                    tenancy_config,
                    ui_config,
                    args.e2e_output_data_path.as_ref(),
                )
                .await?;

            let flightsql_server = crate::flightsql_server::FlightSqlServer::new(
                address,
                c.flightsql_port,
                final_catalog.clone(),
            )
            .await;

            if let Some(e2e_output_data_path) = args.e2e_output_data_path {
                let e2e_file_content = format!(
                    "http://{}\nhttp://{}",
                    local_addr,
                    flightsql_server.local_addr()
                );

                std::fs::write(e2e_output_data_path, e2e_file_content).unwrap();
            }

            // System services are built from the special catalog that contains the admin
            // subject. Thus all services that require authorization are granted full access
            // to all resources.
            //
            // TODO: Granting admin access to all system services is a security threat. We
            // should consider to instead propagate the auth info of the user who triggered
            // some system flow alongside all actions to enforce proper authorization.
            let system_catalog = CatalogBuilder::new_chained(&final_catalog)
                .add_value(server_account_subject)
                .build();

            let task_agent = system_catalog
                .get_one::<dyn kamu_task_system::TaskAgent>()
                .unwrap();

            let flow_agent = system_catalog
                .get_one::<dyn kamu_flow_system::FlowAgent>()
                .unwrap();

            let outbox_agent = system_catalog
                .get_one::<messaging_outbox::OutboxAgent>()
                .unwrap();

            info!(
                http_endpoint = format!("http://{}", local_addr),
                flightsql_endpoint = format!("flightsql://{}", flightsql_server.local_addr()),
                "Serving traffic"
            );

            let server_run_fut: Pin<Box<dyn Future<Output = _>>> =
                if let Some(shutdown_notify) = maybe_shutdown_notify {
                    let server_with_graceful_shutdown = async move {
                        tokio::select! {
                            _ = shutdown_requested => {}
                            _ = shutdown_notify.notified() => {}
                        }
                    };

                    Box::pin(async move {
                        http_server
                            .with_graceful_shutdown(server_with_graceful_shutdown)
                            .await
                    })
                } else {
                    Box::pin(http_server.into_future())
                };

            // Run phase
            // TODO: PERF: Do we need to spawn these into separate tasks?
            tokio::select! {
                res = server_run_fut => { res.int_err() },
                res = flightsql_server.run() => { res.int_err() },
                res = task_agent.run() => { res.int_err() },
                res = flow_agent.run() => { res.int_err() },
                res = outbox_agent.run() => { res.int_err() },
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn init_observability() -> observability::init::Guard {
    observability::init::auto(
        observability::config::Config::from_env_with_prefix("KAMU_OTEL_")
            .with_service_name(BINARY_NAME)
            .with_service_version(VERSION)
            .with_default_log_levels(DEFAULT_RUST_LOG),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn load_config(path: Option<&PathBuf>) -> Result<config::ApiServerConfig, InternalError> {
    use figment::providers::Format;

    let mut figment = figment::Figment::from(figment::providers::Serialized::defaults(
        config::ApiServerConfig::default(),
    ));

    if let Some(path) = path {
        if !path.is_file() {
            return InternalError::bail(format!("Config file '{}' not found", path.display()));
        }

        figment = figment.merge(figment::providers::Yaml::file(path));
    };

    figment
        .merge(
            figment::providers::Env::prefixed("KAMU_API_SERVER_CONFIG_")
                .split("__")
                .lowercase(false),
        )
        .extract()
        .int_err()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_dependencies(
    config: config::ApiServerConfig,
    repo_url: &Url,
    tenancy_config: TenancyConfig,
    local_dir: &Path,
) -> Result<CatalogBuilder, InternalError> {
    // TODO: Revisit this ugly way to get metrics
    let s3_metrics_catalog = CatalogBuilder::new()
        .add_value(S3Metrics::new())
        .bind::<dyn MetricsProvider, S3Metrics>()
        .build();
    let s3_metrics = s3_metrics_catalog.get_one::<S3Metrics>().unwrap();

    let mut b = CatalogBuilder::new_chained(&s3_metrics_catalog);

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

    b.add_value(config.protocol.ipfs.into_gateway_config());
    b.add_value(kamu::utils::ipfs_wrapper::IpfsClient::default());

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

    b.add::<kamu::DatasetChangesServiceImpl>();

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
    b.add::<kamu::ResetPlannerImpl>();
    b.add::<kamu::ResetExecutorImpl>();
    b.add::<kamu::RemoteStatusServiceImpl>();

    b.add::<kamu_datasets_services::AppendDatasetMetadataBatchUseCaseImpl>();
    b.add::<kamu_datasets_services::CommitDatasetEventUseCaseImpl>();
    b.add::<kamu::CompactDatasetUseCaseImpl>();
    b.add::<kamu_datasets_services::CreateDatasetFromSnapshotUseCaseImpl>();
    b.add::<kamu_datasets_services::CreateDatasetUseCaseImpl>();
    b.add::<kamu_datasets_services::DeleteDatasetUseCaseImpl>();
    b.add::<kamu_datasets_services::EditDatasetUseCaseImpl>();
    b.add::<kamu::GetDatasetDownstreamDependenciesUseCaseImpl>();
    b.add::<kamu::GetDatasetUpstreamDependenciesUseCaseImpl>();
    b.add::<kamu::PullDatasetUseCaseImpl>();
    b.add::<kamu::PushDatasetUseCaseImpl>();
    b.add::<kamu_datasets_services::RenameDatasetUseCaseImpl>();
    b.add::<kamu::ResetDatasetUseCaseImpl>();
    b.add::<kamu::SetWatermarkUseCaseImpl>();
    b.add::<kamu::VerifyDatasetUseCaseImpl>();
    b.add::<kamu_datasets_services::ViewDatasetUseCaseImpl>();

    b.add_builder(
        messaging_outbox::OutboxImmediateImpl::builder()
            .with_consumer_filter(messaging_outbox::ConsumerFilter::ImmediateConsumers),
    );
    b.add::<messaging_outbox::OutboxTransactionalImpl>();
    b.add::<messaging_outbox::OutboxDispatchingImpl>();
    b.bind::<dyn messaging_outbox::Outbox, messaging_outbox::OutboxDispatchingImpl>();
    b.add::<messaging_outbox::OutboxAgent>();
    b.add::<messaging_outbox::OutboxAgentMetrics>();

    b.add::<kamu_datasets_services::DatasetEntryServiceImpl>();
    b.add::<kamu_datasets_services::DependencyGraphServiceImpl>();
    b.add::<kamu_datasets_services::DatasetEntryIndexer>();
    b.add::<kamu_datasets_services::DependencyGraphIndexer>();

    messaging_outbox::register_message_dispatcher::<kamu_datasets::DatasetLifecycleMessage>(
        &mut b,
        kamu_datasets::MESSAGE_PRODUCER_KAMU_DATASET_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_task_system::TaskProgressMessage>(
        &mut b,
        kamu_task_system::MESSAGE_PRODUCER_KAMU_TASK_AGENT,
    );
    messaging_outbox::register_message_dispatcher::<
        kamu_flow_system::FlowConfigurationUpdatedMessage,
    >(
        &mut b,
        kamu_flow_system_services::MESSAGE_PRODUCER_KAMU_FLOW_CONFIGURATION_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_flow_system::FlowTriggerUpdatedMessage>(
        &mut b,
        kamu_flow_system_services::MESSAGE_PRODUCER_KAMU_FLOW_TRIGGER_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_flow_system::FlowProgressMessage>(
        &mut b,
        kamu_flow_system_services::MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_accounts::AccessTokenLifecycleMessage>(
        &mut b,
        kamu_accounts::MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
    );
    messaging_outbox::register_message_dispatcher::<kamu_accounts::AccountLifecycleMessage>(
        &mut b,
        kamu_accounts::MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
    );

    b.add_value(messaging_outbox::OutboxConfig::new(
        Duration::seconds(config.outbox.awaiting_step_secs.unwrap()),
        config.outbox.batch_size.unwrap(),
    ));

    let task_agent_config = config.flow_system.task_agent.unwrap();
    b.add_value(kamu_task_system::TaskAgentConfig::new(Duration::seconds(
        task_agent_config.task_checking_interval_secs.unwrap(),
    )));
    kamu_task_system_services::register_dependencies(&mut b);

    let flow_agent_config = config.flow_system.flow_agent.unwrap();
    b.add_value(kamu_flow_system::FlowAgentConfig::new(
        Duration::seconds(flow_agent_config.awaiting_step_secs.unwrap()),
        Duration::seconds(flow_agent_config.mandatory_throttling_period_secs.unwrap()),
    ));

    kamu_flow_system_services::register_dependencies(&mut b);

    b.add::<kamu_accounts_services::AuthenticationServiceImpl>();
    b.add::<kamu_accounts_services::AccessTokenServiceImpl>();
    b.add::<kamu_accounts_services::PredefinedAccountsRegistrator>();

    kamu_adapter_auth_oso_rebac::register_dependencies(&mut b);

    b.add::<kamu_auth_rebac_services::RebacDatasetLifecycleMessageConsumer>();
    b.add::<kamu_auth_rebac_services::RebacIndexer>();

    b.add::<odf::dataset::DummyOdfServerAccessTokenResolver>();

    configure_repository(&mut b, repo_url, &config.repo, &s3_metrics).await;

    b.add::<kamu_accounts_services::LoginPasswordAuthProvider>();
    b.add::<kamu_accounts_services::AccountServiceImpl>();

    let mut need_to_add_default_predefined_accounts_config = true;

    match tenancy_config {
        TenancyConfig::SingleTenant => {
            b.add_value(kamu_accounts::PredefinedAccountsConfig::single_tenant());
            need_to_add_default_predefined_accounts_config = false;
        }
        TenancyConfig::MultiTenant => {
            for provider in config.auth.providers {
                match provider {
                    config::AuthProviderConfig::Github(github_config) => {
                        b.add::<kamu_adapter_oauth::OAuthGithub>();

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

    b.add_value(kamu::domain::ServerUrlConfig::new(
        kamu::domain::Protocols {
            base_url_platform: config.url.base_url_platform,
            base_url_rest: config.url.base_url_rest,
            base_url_flightsql: config.url.base_url_flightsql,
        },
    ));

    let maybe_jwt_secret = if !config.auth.jwt_secret.is_empty() {
        Some(config.auth.jwt_secret)
    } else {
        None
    };

    b.add_value(kamu_accounts::JwtAuthenticationConfig::new(
        maybe_jwt_secret,
    ));

    b.add_value(kamu_adapter_http::FileUploadLimitConfig::new_in_mb(
        config.upload_repo.max_file_size_mb,
    ));

    match config.upload_repo.storage {
        config::UploadRepoStorageConfig::Local => {
            b.add::<kamu_adapter_http::UploadServiceLocal>();
        }
        config::UploadRepoStorageConfig::S3(s3_config) => {
            let s3_upload_direct_url = Url::parse(&s3_config.bucket_s3_url).unwrap();
            let s3_context = S3Context::from_url(&s3_upload_direct_url)
                .await
                .with_metrics(s3_metrics);

            b.add_builder(
                kamu_adapter_http::UploadServiceS3::builder().with_s3_upload_context(s3_context),
            );
            b.bind::<dyn kamu_adapter_http::UploadService, kamu_adapter_http::UploadServiceS3>();
        }
    }

    let dataset_env_vars_config = &config.dataset_env_vars;
    match dataset_env_vars_config.encryption_key.as_ref() {
        None => {
            match dataset_env_vars_config.enabled.as_ref() {
                None => {
                    error!("Dataset env vars configuration is missing. Feature will be disabled");
                }
                Some(true) => panic!("Dataset env vars encryption key is required"),
                _ => {}
            }
            b.add::<kamu_datasets_services::DatasetKeyValueServiceSysEnv>();
            b.add::<kamu_datasets_services::DatasetEnvVarServiceNull>();
        }
        Some(encryption_key) => {
            if let Some(enabled) = &dataset_env_vars_config.enabled
                && !enabled
            {
                warn!("Dataset env vars feature will be disabled");
                b.add::<kamu_datasets_services::DatasetKeyValueServiceSysEnv>();
                b.add::<kamu_datasets_services::DatasetEnvVarServiceNull>();
            } else {
                assert!(
                    kamu_datasets::DatasetEnvVar::try_asm_256_gcm_from_str(encryption_key).is_ok(),
                    "Invalid dataset env var encryption key. Key must be a 32-character \
                     alphanumeric string",
                );
                b.add::<kamu_datasets_services::DatasetKeyValueServiceImpl>();
                b.add::<kamu_datasets_services::DatasetEnvVarServiceImpl>();
            }
        }
    }
    b.add_value(config.dataset_env_vars);

    b.add::<database_common::DatabaseTransactionRunner>();

    b.add::<kamu_auth_rebac_services::RebacServiceImpl>();
    b.add_value(kamu_auth_rebac_services::DefaultAccountProperties { is_admin: false });
    b.add_value(kamu_auth_rebac_services::DefaultDatasetProperties {
        allows_anonymous_read: false,
        allows_public_read: false,
    });

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
    b.add::<kamu_search_services::SearchServiceLocalImpl>();

    if let Some(config::SearchConfig {
        indexer,
        embeddings_chunker,
        embeddings_encoder,
        vector_repo,
    }) = config.search
    {
        b.add::<kamu_search_services::SearchServiceLocalIndexer>();
        b.add_value(kamu_search_services::SearchServiceLocalIndexerConfig {
            clear_on_start: indexer.unwrap_or_default().clear_on_start,
        });

        match embeddings_chunker.unwrap_or_default() {
            config::EmbeddingsChunkerConfig::Simple(cfg) => {
                let d = config::EmbeddingsChunkerConfigSimple::default();
                b.add::<kamu_search_services::EmbeddingsChunkerSimple>();
                b.add_value(kamu_search_services::EmbeddingsChunkerConfigSimple {
                    split_sections: cfg.split_sections.or(d.split_sections).unwrap(),
                    split_paragraphs: cfg.split_paragraphs.or(d.split_paragraphs).unwrap(),
                });
            }
        }

        match embeddings_encoder {
            config::EmbeddingsEncoderConfig::OpenAi(cfg) => {
                let d = config::EmbeddingsEncoderConfigOpenAi::default();
                b.add::<kamu_search_openai::EmbeddingsEncoderOpenAi>();
                b.add_value(kamu_search_openai::EmbeddingsEncoderConfigOpenAI {
                    url: cfg.url,
                    api_key: Some(cfg.api_key.into()),
                    model_name: cfg.model_name.or(d.model_name).unwrap(),
                    dimensions: cfg.dimensions.or(d.dimensions).unwrap(),
                });
            }
        }

        match vector_repo {
            config::VectorRepoConfig::Qdrant(cfg) => {
                let d = config::VectorRepoConfigQdrant::default();
                b.add::<kamu_search_qdrant::VectorRepositoryQdrant>();
                b.add_value(kamu_search_qdrant::VectorRepositoryConfigQdrant {
                    url: cfg.url,
                    collection_name: cfg.collection_name.or(d.collection_name).unwrap(),
                    dimensions: cfg.dimensions.or(d.dimensions).unwrap(),
                });
            }
        }
    }
    //

    Ok(b)
}

async fn configure_repository(
    b: &mut CatalogBuilder,
    repo_url: &Url,
    config: &config::RepoConfig,
    s3_metrics: &Arc<S3Metrics>,
) {
    match repo_url.scheme() {
        "file" => {
            use odf::dataset::DatasetStorageUnitLocalFs;

            let datasets_dir = repo_url.to_file_path().unwrap();

            b.add_builder(DatasetStorageUnitLocalFs::builder().with_root(datasets_dir.clone()));
            b.bind::<dyn odf::DatasetStorageUnit, DatasetStorageUnitLocalFs>();
            b.bind::<dyn odf::DatasetStorageUnitWriter, DatasetStorageUnitLocalFs>();

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

            let metadata_cache_local_fs_path = config
                .caching
                .metadata_local_fs_cache_path
                .clone()
                .map(Arc::new);

            b.add_builder(
                DatasetStorageUnitS3::builder()
                    .with_s3_context(s3_context.clone())
                    .with_metadata_cache_local_fs_path(metadata_cache_local_fs_path),
            );
            b.bind::<dyn odf::DatasetStorageUnit, DatasetStorageUnitS3>();
            b.bind::<dyn odf::DatasetStorageUnitWriter, DatasetStorageUnitS3>();

            let allow_http = repo_url.scheme() == "s3+http";
            b.add_value(kamu::ObjectStoreBuilderS3::new(s3_context, allow_http))
                .bind::<dyn kamu::domain::ObjectStoreBuilder, kamu::ObjectStoreBuilderS3>();

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

async fn initialize_components(
    catalog: &dill::Catalog,
    server_account_subject: kamu_accounts::CurrentAccountSubject,
) -> Result<(), InternalError> {
    let logged_catalog = dill::CatalogBuilder::new_chained(catalog)
        .add_value(server_account_subject)
        .build();

    init_on_startup::run_startup_jobs(&logged_catalog)
        .await
        .int_err()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
