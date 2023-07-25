// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::{Path, PathBuf};

use dill::{builder_for, CatalogBuilder};
use internal_error::*;
use kamu::domain::CurrentAccountSubject;
use kamu::utils::smart_transfer_protocol::SmartTransferProtocolClient;
use tracing::info;
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LOGGING_CONFIG: &str = "info,tower_http=trace";

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(matches: clap::ArgMatches) -> Result<(), InternalError> {
    init_logging();

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

    let catalog = init_dependencies(&repo_url, local_dir.path()).await.build();

    match matches.subcommand() {
        Some(("gql", sub)) => match sub.subcommand() {
            Some(("schema", _)) => {
                println!("{}", kamu_adapter_graphql::schema(catalog).sdl());
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
            let server = crate::http_server::build_server(
                sub.get_one("address").map(|a| *a),
                sub.get_one("http-port").map(|p| *p),
                catalog.clone(),
            );

            tracing::info!(
                http_endpoint = format!("http://{}", server.local_addr()),
                "Serving traffic"
            );

            let task_executor = catalog
                .get_one::<dyn kamu_task_system_inmem::domain::TaskExecutor>()
                .unwrap();

            tokio::select! {
                res = server => { res.int_err() },
                res = task_executor.run() => { res.int_err() },
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

pub async fn init_dependencies(repo_url: &Url, local_dir: &Path) -> CatalogBuilder {
    let mut b = dill::CatalogBuilder::new();

    // TODO: Improve output multiplexing and cache interface
    let run_info_dir = local_dir.join("run");
    let cache_dir = local_dir.join("cache");
    let remote_repos_dir = local_dir.join("repos");
    std::fs::create_dir_all(&run_info_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&remote_repos_dir).unwrap();

    // TODO: replace with other means of emitting current account from HTTP sessions
    let current_account_subject = CurrentAccountSubject::new("kamu");
    b.add_value(current_account_subject);

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

    b.add::<kamu::DatasetFactoryImpl>();
    b.bind::<dyn kamu::domain::DatasetFactory, kamu::DatasetFactoryImpl>();

    b.add::<kamu::ObjectStoreRegistryImpl>();
    b.bind::<dyn kamu::domain::ObjectStoreRegistry, kamu::ObjectStoreRegistryImpl>();

    b.add::<kamu::RemoteAliasesRegistryImpl>();
    b.bind::<dyn kamu::domain::RemoteAliasesRegistry, kamu::RemoteAliasesRegistryImpl>();

    b.add_builder(
        builder_for::<kamu::RemoteRepositoryRegistryImpl>()
            .with_repos_dir(remote_repos_dir.clone()),
    );
    b.bind::<dyn kamu::domain::RemoteRepositoryRegistry, kamu::RemoteRepositoryRegistryImpl>();

    b.add::<kamu_adapter_http::SmartTransferProtocolClientWs>();
    b.bind::<dyn SmartTransferProtocolClient, kamu_adapter_http::SmartTransferProtocolClientWs>();

    b.add_builder(
        builder_for::<kamu::EngineProvisionerLocal>().with_run_info_dir(run_info_dir.clone()),
    );
    b.bind::<dyn kamu::domain::EngineProvisioner, kamu::EngineProvisionerLocal>();

    b.add_builder(
        builder_for::<kamu::IngestServiceImpl>()
            .with_run_info_dir(run_info_dir.clone())
            .with_cache_dir(cache_dir.clone()),
    );
    b.bind::<dyn kamu::domain::IngestService, kamu::IngestServiceImpl>();

    b.add::<kamu::TransformServiceImpl>();
    b.bind::<dyn kamu::domain::TransformService, kamu::TransformServiceImpl>();

    b.add::<kamu::SyncServiceImpl>();
    b.bind::<dyn kamu::domain::SyncService, kamu::SyncServiceImpl>();

    b.add::<kamu::VerificationServiceImpl>();
    b.bind::<dyn kamu::domain::VerificationService, kamu::VerificationServiceImpl>();

    b.add::<kamu::PullServiceImpl>();
    b.bind::<dyn kamu::domain::PullService, kamu::PullServiceImpl>();

    b.add::<kamu::QueryServiceImpl>();
    b.bind::<dyn kamu::domain::QueryService, kamu::QueryServiceImpl>();

    // TODO: Externalize configuration
    b.add_value(kamu::IpfsGateway {
        url: Url::parse("http://localhost:8080").unwrap(),
        pre_resolve_dnslink: true,
    });
    b.add_value(kamu::utils::ipfs_wrapper::IpfsClient::default());

    b.add::<kamu_task_system_inmem::TaskSchedulerInMemory>();
    b.bind::<dyn kamu_task_system_inmem::domain::TaskScheduler, kamu_task_system_inmem::TaskSchedulerInMemory>();

    b.add::<kamu_task_system_inmem::TaskSystemEventStoreInMemory>();
    b.bind::<dyn kamu_task_system_inmem::domain::TaskSystemEventStore, kamu_task_system_inmem::TaskSystemEventStoreInMemory>();

    b.add::<kamu_task_system_inmem::TaskExecutorInMemory>();
    b.bind::<dyn kamu_task_system_inmem::domain::TaskExecutor, kamu_task_system_inmem::TaskExecutorInMemory>();

    match repo_url.scheme() {
        "file" => {
            let datasets_dir = repo_url.to_file_path().unwrap();

            b.add_builder(
                builder_for::<kamu::DatasetRepositoryLocalFs>()
                    .with_root(datasets_dir.clone())
                    .with_multi_tenant(false),
            );
            b.bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryLocalFs>();

            b.add::<kamu::ObjectStoreBuilderLocalFs>();
            b.bind::<dyn kamu::domain::ObjectStoreBuilder, kamu::ObjectStoreBuilderLocalFs>();
        }
        "s3" | "s3+http" | "s3+https" => {
            let s3_context = kamu::utils::s3_context::S3Context::from_url(&repo_url).await;

            b.add_builder(
                builder_for::<kamu::DatasetRepositoryS3>().with_s3_context(s3_context.clone()),
            );
            b.bind::<dyn kamu::domain::DatasetRepository, kamu::DatasetRepositoryS3>();

            b.add_value(kamu::ObjectStoreBuilderS3::new(s3_context, false))
                .bind::<dyn kamu::domain::ObjectStoreBuilder, kamu::ObjectStoreBuilderS3>();
        }
        _ => panic!("Unsupported repository scheme: {}", repo_url.scheme()),
    }

    b
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
