// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dill::{CatalogBuilder, Component};
use internal_error::*;
use kamu::domain::{Protocols, ServerUrlConfig, SystemTimeSourceDefault};
use kamu::utils::s3_context::S3Context;
use kamu_accounts::{CurrentAccountSubject, JwtAuthenticationConfig, PredefinedAccountsConfig};
use kamu_accounts_services::LoginPasswordAuthProvider;
use kamu_adapter_http::{
    FileUploadLimitConfig,
    UploadService,
    UploadServiceLocal,
    UploadServiceS3,
};
use kamu_adapter_oauth::GithubAuthenticationConfig;
use opendatafabric::{AccountID, AccountName};
use tracing::info;
use url::Url;

use crate::config::{
    ApiServerConfig,
    AuthProviderConfig,
    RepoConfig,
    UploadRepoStorageConfig,
    ACCOUNT_KAMU,
};
use crate::{
    configure_database_components,
    configure_in_memory_components,
    connect_database_initially,
    spawn_password_refreshing_job,
    try_build_db_connection_settings,
};

/////////////////////////////////////////////////////////////////////////////////////////

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LOGGING_CONFIG: &str = "info,tower_http=trace";

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(matches: clap::ArgMatches, config: ApiServerConfig) -> Result<(), InternalError> {
    init_logging();

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

    let multi_tenant = matches.get_flag("multi-tenant") || repo_url.scheme() != "file";

    let local_dir = tempfile::tempdir().unwrap();

    info!(
        version = VERSION,
        args = ?std::env::args().collect::<Vec<_>>(),
        repo_url = %repo_url,
        local_dir = %local_dir.path().display(),
        config = ?config,
        "Initializing {BINARY_NAME}",
    );

    let kamu_account_name = AccountName::new_unchecked(ACCOUNT_KAMU);
    let server_account_subject = CurrentAccountSubject::logged(
        AccountID::new_seeded_ed25519(kamu_account_name.as_bytes()),
        kamu_account_name,
        true,
    );
    let dependencies_graph_repository = prepare_dependencies_graph_repository(
        server_account_subject.clone(),
        &repo_url,
        multi_tenant,
        &config,
    )
    .await;

    let db_config = config.database.clone();

    let catalog = init_dependencies(config, &repo_url, multi_tenant, local_dir.path())
        .await
        .add_value(dependencies_graph_repository)
        .bind::<dyn kamu::domain::DependencyGraphRepository, kamu::DependencyGraphRepositoryInMemory>()
        .build();

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

    initialize_components(&final_catalog, server_account_subject.clone()).await;

    match matches.subcommand() {
        Some(("gql", sub)) => match sub.subcommand() {
            Some(("schema", _)) => {
                println!("{}", kamu_adapter_graphql::schema().sdl());
                Ok(())
            }
            Some(("query", qsub)) => {
                let result = crate::gql_server::gql_query(
                    qsub.get_one("query").map(String::as_str).unwrap(),
                    qsub.get_flag("full"),
                    final_catalog,
                )
                .await;
                print!("{}", result);
                Ok(())
            }
            _ => unimplemented!(),
        },
        Some(("run", sub)) => {
            let address = sub
                .get_one::<std::net::IpAddr>("address")
                .copied()
                .unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1).into());

            // API servers are built from the regular catalog
            // that does not contain any auth subject, thus they will rely on
            // their own middlewares to authenticate per request / session and execute
            // all processing in the user context.
            let http_server = crate::http_server::build_server(
                address,
                sub.get_one("http-port").copied(),
                final_catalog.clone(),
                multi_tenant,
            );

            let flightsql_server = crate::flightsql_server::FlightSqlServer::new(
                address,
                sub.get_one("flightsql-port").copied(),
                final_catalog.clone(),
            )
            .await;

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

            let task_executor = system_catalog
                .get_one::<dyn kamu_task_system_inmem::domain::TaskExecutor>()
                .unwrap();

            let flow_service = system_catalog
                .get_one::<dyn kamu_flow_system_inmem::domain::FlowService>()
                .unwrap();

            let now = system_catalog
                .get_one::<dyn kamu::domain::SystemTimeSource>()
                .unwrap()
                .now();

            info!(
                http_endpoint = format!("http://{}", http_server.local_addr()),
                flightsql_endpoint = format!("flightsql://{}", flightsql_server.local_addr()),
                "Serving traffic"
            );

            tokio::select! {
                res = http_server => { res.int_err() },
                res = flightsql_server.run() => { res.int_err() },
                res = task_executor.run() => { res.int_err() },
                res = flow_service.run(now) => { res.int_err() },
            }
        }
        _ => unimplemented!(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn init_logging() {
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_log::LogTracer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::EnvFilter;

    // Logging may be already initialized when running under tests
    if tracing::dispatcher::has_been_set() {
        return;
    }

    // Use configuration from RUST_LOG env var if provided
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(DEFAULT_LOGGING_CONFIG));

    // TODO: Use non-blocking writer?
    // Configure Bunyan JSON formatter
    let formatting_layer = BunyanFormattingLayer::new(BINARY_NAME.to_owned(), std::io::stdout);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // Redirect all standard logging to tracing events
    LogTracer::init().expect("Failed to set LogTracer");

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

/////////////////////////////////////////////////////////////////////////////////////////

pub fn load_config(path: Option<&PathBuf>) -> Result<ApiServerConfig, InternalError> {
    use figment::providers::Format;

    let mut figment = figment::Figment::from(figment::providers::Serialized::defaults(
        ApiServerConfig::default(),
    ));

    if let Some(path) = path {
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

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Get rid of this
pub async fn prepare_dependencies_graph_repository(
    current_account_subject: CurrentAccountSubject,
    repo_url: &Url,
    multi_tenant: bool,
    config: &ApiServerConfig,
) -> kamu::DependencyGraphRepositoryInMemory {
    // Construct a special catalog just to create 1 object, but with a repository
    // bound to API server command line user. It also should be authorized to access
    // any dataset.

    let mut b = CatalogBuilder::new();

    configure_repository(&mut b, repo_url, multi_tenant, &config.repo).await;

    let special_catalog = b
        .add::<SystemTimeSourceDefault>()
        .add::<event_bus::EventBus>()
        .add_value(current_account_subject)
        .add::<kamu::domain::auth::AlwaysHappyDatasetActionAuthorizer>()
        .add::<kamu::DependencyGraphServiceInMemory>()
        // Don't add its own initializer, leave optional dependency uninitialized
        .build();

    let dataset_repo = special_catalog.get_one().unwrap();

    kamu::DependencyGraphRepositoryInMemory::new(dataset_repo)
}

pub async fn init_dependencies(
    config: ApiServerConfig,
    repo_url: &Url,
    multi_tenant: bool,
    local_dir: &Path,
) -> CatalogBuilder {
    let mut b = dill::CatalogBuilder::new();

    // TODO: Improve output multiplexing and cache interface
    let run_info_dir = local_dir.join("run");
    let cache_dir = local_dir.join("cache");
    let remote_repos_dir = local_dir.join("repos");
    std::fs::create_dir_all(&run_info_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&remote_repos_dir).unwrap();

    b.add_value(kamu::domain::RunInfoDir::new(run_info_dir));
    b.add_value(kamu::domain::CacheDir::new(cache_dir));
    b.add_value(kamu::RemoteReposDir::new(remote_repos_dir));
    b.add::<kamu::domain::SystemTimeSourceDefault>();
    b.add::<event_bus::EventBus>();

    b.add::<container_runtime::ContainerRuntime>();
    b.add_value(container_runtime::ContainerRuntimeConfig {
        runtime: container_runtime::ContainerRuntimeType::Podman,
        network_ns: container_runtime::NetworkNamespaceType::Private,
    });

    // TODO: Externalize config
    b.add_value(kamu::EngineProvisionerLocalConfig {
        max_concurrency: Some(2),
        ..Default::default()
    });
    b.add::<kamu::EngineProvisionerLocal>();

    b.add::<kamu::DatasetFactoryImpl>();
    b.add::<kamu::ObjectStoreRegistryImpl>();
    b.add::<kamu::RemoteAliasesRegistryImpl>();

    // TODO: initialize graph dependencies when starting API server
    b.add::<kamu::DependencyGraphServiceInMemory>();

    b.add::<kamu::DatasetOwnershipServiceInMemory>();
    b.add::<kamu::DatasetOwnershipServiceInMemoryStateInitializer>();

    b.add::<kamu::DatasetChangesServiceImpl>();

    b.add::<kamu::RemoteRepositoryRegistryImpl>();

    b.add::<kamu_adapter_http::SmartTransferProtocolClientWs>();

    b.add::<kamu::DataFormatRegistryImpl>();

    b.add::<kamu::FetchService>();
    b.add::<kamu::PollingIngestServiceImpl>();
    b.add::<kamu::PushIngestServiceImpl>();
    b.add::<kamu::TransformServiceImpl>();
    b.add::<kamu::SyncServiceImpl>();
    b.add::<kamu::CompactionServiceImpl>();
    b.add::<kamu::VerificationServiceImpl>();
    b.add::<kamu::PullServiceImpl>();
    b.add::<kamu::QueryServiceImpl>();

    // TODO: Externalize configuration
    b.add_value(kamu::IpfsGateway {
        url: Url::parse("http://localhost:8080").unwrap(),
        pre_resolve_dnslink: true,
    });
    b.add_value(kamu::utils::ipfs_wrapper::IpfsClient::default());

    b.add::<kamu_task_system_services::TaskSchedulerImpl>();
    b.add::<kamu_task_system_services::TaskExecutorImpl>();

    b.add::<kamu_flow_system_services::FlowConfigurationServiceImpl>();
    b.add::<kamu_flow_system_services::FlowServiceImpl>();
    b.add_value(kamu_flow_system_inmem::domain::FlowServiceRunConfig::new(
        chrono::Duration::try_seconds(1).unwrap(),
        chrono::Duration::try_minutes(1).unwrap(),
    ));

    b.add::<kamu_accounts_services::AuthenticationServiceImpl>();
    b.add::<kamu_accounts_services::AccessTokenServiceImpl>();
    b.add::<kamu_accounts_services::PredefinedAccountsRegistrator>();

    b.add::<kamu_adapter_auth_oso::KamuAuthOso>();
    b.add::<kamu_adapter_auth_oso::OsoDatasetAuthorizer>();
    b.add::<kamu::domain::auth::DummyOdfServerAccessTokenResolver>();

    configure_repository(&mut b, repo_url, multi_tenant, &config.repo).await;

    b.add::<LoginPasswordAuthProvider>();

    let mut need_to_add_default_predefined_accounts_config = true;

    if !multi_tenant {
        b.add_value(PredefinedAccountsConfig::single_tenant());
        need_to_add_default_predefined_accounts_config = false
    } else {
        for provider in config.auth.providers {
            match provider {
                AuthProviderConfig::Github(github_config) => {
                    b.add::<kamu_adapter_oauth::OAuthGithub>();

                    b.add_value(GithubAuthenticationConfig::new(
                        github_config.client_id,
                        github_config.client_secret,
                    ));
                }
                AuthProviderConfig::Password(prov) => {
                    b.add_value(PredefinedAccountsConfig {
                        predefined: prov.accounts,
                    });
                    need_to_add_default_predefined_accounts_config = false
                }
            }
        }
    }

    if need_to_add_default_predefined_accounts_config {
        b.add_value(PredefinedAccountsConfig::default());
    }

    b.add_value(ServerUrlConfig::new(Protocols {
        base_url_platform: config.url.base_url_platform,
        base_url_rest: config.url.base_url_rest,
        base_url_flightsql: config.url.base_url_flightsql,
    }));

    let maybe_jwt_secret = if !config.auth.jwt_secret.is_empty() {
        Some(config.auth.jwt_secret)
    } else {
        None
    };

    b.add_value(JwtAuthenticationConfig::new(maybe_jwt_secret));

    b.add_value(FileUploadLimitConfig::new_in_mb(
        config.upload_repo.max_file_size_mb,
    ));

    match config.upload_repo.storage {
        UploadRepoStorageConfig::Local => {
            b.add::<UploadServiceLocal>();
        }
        UploadRepoStorageConfig::S3(s3_config) => {
            let s3_upload_direct_url = url::Url::parse(&s3_config.bucket_s3_url).unwrap();
            b.add_builder(
                UploadServiceS3::builder()
                    .with_s3_upload_context(S3Context::from_url(&s3_upload_direct_url).await),
            );
            b.bind::<dyn UploadService, UploadServiceS3>();
        }
    }

    b.add::<database_common::DatabaseTransactionRunner>();

    let maybe_db_connection_settings = try_build_db_connection_settings(&config.database);
    if let Some(db_connection_settings) = maybe_db_connection_settings {
        configure_database_components(&mut b, &config.database, db_connection_settings);
    } else {
        configure_in_memory_components(&mut b);
    };

    b
}

async fn configure_repository(
    b: &mut CatalogBuilder,
    repo_url: &Url,
    multi_tenant: bool,
    config: &RepoConfig,
) {
    match repo_url.scheme() {
        "file" => {
            let datasets_dir = repo_url.to_file_path().unwrap();

            b.add_builder(
                kamu::DatasetRepositoryLocalFs::builder()
                    .with_root(datasets_dir.clone())
                    .with_multi_tenant(multi_tenant),
            );
            b.bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryLocalFs>();

            b.add::<kamu::ObjectStoreBuilderLocalFs>();
        }
        "s3" | "s3+http" | "s3+https" => {
            let s3_context = kamu::utils::s3_context::S3Context::from_url(repo_url).await;

            if config.caching.registry_cache_enabled {
                b.add::<kamu::S3RegistryCache>();
            }

            let metadata_cache_local_fs_path = config
                .caching
                .metadata_local_fs_cache_path
                .clone()
                .map(Arc::new);

            b.add_builder(
                kamu::DatasetRepositoryS3::builder()
                    .with_s3_context(s3_context.clone())
                    .with_multi_tenant(true)
                    .with_metadata_cache_local_fs_path(metadata_cache_local_fs_path),
            )
            .bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryS3>();

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

/////////////////////////////////////////////////////////////////////////////////////////
// Workspace
/////////////////////////////////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////////////////////////////////

async fn initialize_components(
    catalog: &dill::Catalog,
    server_account_subject: CurrentAccountSubject,
) {
    let logged_catalog = dill::CatalogBuilder::new_chained(catalog)
        .add_value(server_account_subject)
        .build();

    database_common::DatabaseTransactionRunner::new(logged_catalog.clone())
        .transactional(|transactional_catalog| async move {
            let registrator = transactional_catalog
                .get_one::<kamu_accounts_services::PredefinedAccountsRegistrator>()
                .unwrap();
            registrator
                .ensure_predefined_accounts_are_registered()
                .await
                .unwrap();

            let initializer = transactional_catalog
                .get_one::<kamu::DatasetOwnershipServiceInMemoryStateInitializer>()
                .unwrap();
            initializer.eager_initialization().await
        })
        .await
        .unwrap();
}

/////////////////////////////////////////////////////////////////////////////////////////
