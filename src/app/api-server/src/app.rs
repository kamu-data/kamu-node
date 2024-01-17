// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::{Path, PathBuf};

use dill::{CatalogBuilder, Component};
use internal_error::*;
use tracing::info;
use url::Url;

use crate::config::{ApiServerConfig, AuthProviderConfig};
use crate::dummy_auth_provider::DummyAuthProvider;

/////////////////////////////////////////////////////////////////////////////////////////

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LOGGING_CONFIG: &str = "info,tower_http=trace";

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(matches: clap::ArgMatches) -> Result<(), InternalError> {
    init_logging();

    let config = load_config(matches.get_one("config"))?;

    let repo_url = if let Some(repo_url) = matches.get_one::<Url>("repo-url").cloned() {
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

    let local_dir = tempfile::tempdir().unwrap();

    info!(
        version = VERSION,
        args = ?std::env::args().collect::<Vec<_>>(),
        repo_url = %repo_url,
        local_dir = %local_dir.path().display(),
        "Initializing {}",
        BINARY_NAME
    );

    let catalog = init_dependencies(config, &repo_url, local_dir.path())
        .await
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
                .map(|a| *a)
                .unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1).into());

            let http_server = crate::http_server::build_server(
                address,
                sub.get_one("http-port").map(|p| *p),
                catalog.clone(),
                should_use_multi_tenancy(&repo_url),
            );

            let flightsql_server = crate::flightsql_server::FlightSqlServer::new(
                address,
                sub.get_one("flightsql-port").map(|p| *p),
                catalog.clone(),
            )
            .await;

            tracing::info!(
                http_endpoint = format!("http://{}", http_server.local_addr()),
                flightsql_endpoint = format!("flightsql://{}", flightsql_server.local_addr()),
                "Serving traffic"
            );

            let task_executor = catalog
                .get_one::<dyn kamu_task_system_inmem::domain::TaskExecutor>()
                .unwrap();

            let flow_service = catalog
                .get_one::<dyn kamu_flow_system_inmem::domain::FlowService>()
                .unwrap();

            let now = catalog
                .get_one::<dyn kamu::domain::SystemTimeSource>()
                .unwrap()
                .now();

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
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(DEFAULT_LOGGING_CONFIG.to_owned()));

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

    let Some(path) = path else {
        return Ok(ApiServerConfig::default());
    };

    figment::Figment::from(figment::providers::Serialized::defaults(
        ApiServerConfig::default(),
    ))
    .merge(figment::providers::Yaml::file(path))
    .merge(figment::providers::Env::prefixed("KAMU_API_SERVER_").lowercase(false))
    .extract()
    .int_err()
}

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_dependencies(
    config: ApiServerConfig,
    repo_url: &Url,
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
    b.add::<kamu::VerificationServiceImpl>();
    b.add::<kamu::PullServiceImpl>();
    b.add::<kamu::QueryServiceImpl>();

    // TODO: Externalize configuration
    b.add_value(kamu::IpfsGateway {
        url: Url::parse("http://localhost:8080").unwrap(),
        pre_resolve_dnslink: true,
    });
    b.add_value(kamu::utils::ipfs_wrapper::IpfsClient::default());

    b.add::<kamu_task_system_inmem::TaskSchedulerInMemory>();
    b.add::<kamu_task_system_inmem::TaskSystemEventStoreInMemory>();
    b.add::<kamu_task_system_inmem::TaskExecutorInMemory>();

    b.add::<kamu_flow_system_inmem::FlowConfigurationServiceInMemory>();
    b.add::<kamu_flow_system_inmem::FlowServiceInMemory>();
    b.add_value(kamu_flow_system_inmem::domain::FlowServiceRunConfig::new(
        chrono::Duration::seconds(1),
    ));
    b.add::<kamu_flow_system_inmem::FlowEventStoreInMem>();
    b.add::<kamu_flow_system_inmem::FlowConfigurationEventStoreInMem>();

    b.add::<kamu::AuthenticationServiceImpl>();
    b.add::<kamu_adapter_auth_oso::KamuAuthOso>();
    b.add::<kamu_adapter_auth_oso::OsoDatasetAuthorizer>();
    b.add::<kamu::domain::auth::DummyOdfServerAccessTokenResolver>();

    match repo_url.scheme() {
        "file" => {
            let datasets_dir = repo_url.to_file_path().unwrap();

            b.add_builder(
                kamu::DatasetRepositoryLocalFs::builder()
                    .with_root(datasets_dir.clone())
                    .with_multi_tenant(false),
            );
            b.bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryLocalFs>();

            b.add::<kamu::ObjectStoreBuilderLocalFs>();
            b.add_value(DummyAuthProvider::new_with_default_account());
            b.bind::<dyn kamu::domain::auth::AuthenticationProvider, DummyAuthProvider>();
        }
        "s3" | "s3+http" | "s3+https" => {
            let s3_context = kamu::utils::s3_context::S3Context::from_url(&repo_url).await;

            b.add_builder(
                kamu::DatasetRepositoryS3::builder()
                    .with_s3_context(s3_context.clone())
                    .with_multi_tenant(true),
            )
            .bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryS3>();

            let allow_http = repo_url.scheme() == "s3+http";
            b.add_value(kamu::ObjectStoreBuilderS3::new(s3_context, allow_http))
                .bind::<dyn kamu::domain::ObjectStoreBuilder, kamu::ObjectStoreBuilderS3>();

            // Default to GitHub auth
            if config.auth.providers.is_empty() {
                b.add::<kamu_adapter_oauth::OAuthGithub>();
            }

            for provider in config.auth.providers {
                match provider {
                    AuthProviderConfig::Github(_) => {
                        b.add::<kamu_adapter_oauth::OAuthGithub>();
                    }
                    AuthProviderConfig::Dummy(prov) => {
                        b.add_value(DummyAuthProvider::new(prov.accounts));
                        b.bind::<dyn kamu::domain::auth::AuthenticationProvider, DummyAuthProvider>();
                    }
                }
            }
        }
        _ => panic!("Unsupported repository scheme: {}", repo_url.scheme()),
    }

    b
}

/////////////////////////////////////////////////////////////////////////////////////////

fn should_use_multi_tenancy(repo_url: &Url) -> bool {
    match repo_url.scheme() {
        "file" => false,
        "s3" | "s3+http" | "s3+https" => true,
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
