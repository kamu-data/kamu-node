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
use kamu_accounts::{
    CurrentAccountSubject,
    JwtAuthenticationConfig,
    PredefinedAccountsConfig,
    DEFAULT_ACCOUNT_NAME,
};
use kamu_adapter_oauth::GithubAuthenticationConfig;
use opendatafabric::AccountID;
use random_names::get_random_name;
use tracing::info;
use url::Url;

use crate::config::{ApiServerConfig, AuthProviderConfig, RepoConfig};
use crate::dummy_auth_provider::DummyAuthProvider;

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

    let logged_account_subject = CurrentAccountSubject::logged(
        AccountID::new_seeded_ed25519(DEFAULT_ACCOUNT_NAME.as_bytes()),
        DEFAULT_ACCOUNT_NAME.clone(),
        true,
    );
    let dependencies_graph_repository = prepare_dependencies_graph_repository(
        logged_account_subject.clone(),
        &repo_url,
        multi_tenant,
        &config,
    )
    .await;

    let catalog = init_dependencies(config, &repo_url, multi_tenant, local_dir.path())
        .await
        .add_value(dependencies_graph_repository)
        .bind::<dyn kamu::domain::DependencyGraphRepository, kamu::DependencyGraphRepositoryInMemory>()
        .build();

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
                    catalog,
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
                catalog.clone(),
                multi_tenant,
            );

            let flightsql_server = crate::flightsql_server::FlightSqlServer::new(
                address,
                sub.get_one("flightsql-port").copied(),
                catalog.clone(),
            )
            .await;

            // System services are built from the special catalog that contains the admin
            // subject. Thus all services that require authorization are granted full access
            // to all resources.
            //
            // TODO: Granting admin access to all system services is a security threat. We
            // should consider to instead propagate the auth info of the user who triggered
            // some system flow alongside all actions to enforce proper authorization.
            let system_catalog = CatalogBuilder::new_chained(&catalog)
                .add_value(logged_account_subject)
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
    b.add_builder(kamu::EngineProvisionerLocal::builder().with_run_info_dir(run_info_dir.clone()))
        .bind::<dyn kamu::domain::EngineProvisioner, kamu::EngineProvisionerLocal>();

    b.add::<kamu::DatasetFactoryImpl>();
    b.add::<kamu::ObjectStoreRegistryImpl>();
    b.add::<kamu::RemoteAliasesRegistryImpl>();

    // TODO: initialize graph dependencies when starting API server
    b.add::<kamu::DependencyGraphServiceInMemory>();

    b.add::<kamu::DatasetChangesServiceImpl>();

    b.add_builder(
        kamu::RemoteRepositoryRegistryImpl::builder().with_repos_dir(remote_repos_dir.clone()),
    )
    .bind::<dyn kamu::domain::RemoteRepositoryRegistry, kamu::RemoteRepositoryRegistryImpl>();

    b.add::<kamu_adapter_http::SmartTransferProtocolClientWs>();

    b.add::<kamu::DataFormatRegistryImpl>();

    b.add_builder(
        kamu::PollingIngestServiceImpl::builder()
            .with_run_info_dir(run_info_dir.clone())
            .with_cache_dir(cache_dir.clone()),
    )
    .bind::<dyn kamu::domain::PollingIngestService, kamu::PollingIngestServiceImpl>();

    b.add_builder(kamu::PushIngestServiceImpl::builder().with_run_info_dir(run_info_dir.clone()))
        .bind::<dyn kamu::domain::PushIngestService, kamu::PushIngestServiceImpl>();

    b.add::<kamu::TransformServiceImpl>();
    b.add::<kamu::SyncServiceImpl>();
    b.add_builder(kamu::CompactingServiceImpl::builder().with_run_info_dir(run_info_dir.clone()))
        .bind::<dyn kamu::domain::CompactingService, kamu::CompactingServiceImpl>();
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
    b.add::<kamu_task_system_inmem::TaskSystemEventStoreInMemory>();

    b.add::<kamu_flow_system_services::FlowConfigurationServiceImpl>();
    b.add::<kamu_flow_system_services::FlowServiceImpl>();
    b.add_value(kamu_flow_system_inmem::domain::FlowServiceRunConfig::new(
        chrono::Duration::try_seconds(1).unwrap(),
        chrono::Duration::try_minutes(1).unwrap(),
    ));
    b.add::<kamu_flow_system_inmem::FlowEventStoreInMem>();
    b.add::<kamu_flow_system_inmem::FlowConfigurationEventStoreInMem>();

    b.add::<kamu_accounts_services::AuthenticationServiceImpl>();
    b.add::<kamu_adapter_auth_oso::KamuAuthOso>();
    b.add::<kamu_adapter_auth_oso::OsoDatasetAuthorizer>();
    b.add::<kamu::domain::auth::DummyOdfServerAccessTokenResolver>();

    configure_repository(&mut b, repo_url, multi_tenant, &config.repo).await;

    b.add::<DummyAuthProvider>();
    // TODO: Temporarily using in-mem
    b.add::<kamu_accounts_inmem::AccountRepositoryInMemory>();

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
                AuthProviderConfig::Dummy(prov) => {
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

    // TODO: Use JwtAuthenticationConfig::new()
    //       update after https://github.com/kamu-data/kamu-cli/pull/623
    let jwt_secret = if !config.auth.jwt_token.is_empty() {
        config.auth.jwt_token
    } else {
        get_random_name(None, 64)
    };

    b.add_value(JwtAuthenticationConfig { jwt_secret });

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
