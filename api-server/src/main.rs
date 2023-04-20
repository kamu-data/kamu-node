mod cli_parser;

use std::path::{Path, PathBuf};

use dill::CatalogBuilder;
use kamu::domain;
use kamu::domain::DatasetRepository;
use kamu::infra;
use kamu::infra::utils::s3_context::S3Context;
use kamu::infra::DatasetRepositoryS3;
use tracing::info;
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LOGGING_CONFIG: &str = "info,tower_http=trace";

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum RunMode {
    LocalWorkspace,
    RemoteS3Url,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    init_logging();

    let matches = cli_parser::cli(BINARY_NAME, VERSION).get_matches();

    let repo_url = matches.get_one::<Url>("repo-url").cloned();
    let local_repo_path = matches.get_one::<PathBuf>("local-repo").cloned();

    let catalog = if let (Some(_), Some(_)) = (&repo_url, &local_repo_path) {
        panic!("Cannot use '--repo-url' and '--local-repo' at the same time.");
    } else if let (Some(repo_url), None) = (&repo_url, &local_repo_path) {
        info!(
            version = VERSION,
            args = ?std::env::args().collect::<Vec<_>>(),
            repo_url = %repo_url,
            "Initializing (remote storage) {}",
            BINARY_NAME
        );

        let mut b = init_dependencies(RunMode::RemoteS3Url);

        let (endpoint, bucket, key_prefix) = S3Context::split_url(&repo_url);
        let s3_context = S3Context::from_items(endpoint.clone(), bucket, key_prefix).await;
        b.add_value(DatasetRepositoryS3::new(s3_context, endpoint.unwrap()))
            .bind::<dyn DatasetRepository, DatasetRepositoryS3>();

        b
    } else {
        let local_repo = local_repo_path.unwrap_or_else(|| find_workspace());

        info!(
            version = VERSION,
            args = ?std::env::args().collect::<Vec<_>>(),
            local_repo = %local_repo.display(),
            "Initializing (local storage) {}",
            BINARY_NAME
        );

        let mut b = init_dependencies(RunMode::LocalWorkspace);

        let workspace_layout = infra::WorkspaceLayout::new(&local_repo);
        b.add_value(workspace_layout.clone());

        init_from_local_workspace(&workspace_layout);

        b
    }
    .build();

    match matches.subcommand() {
        Some(("gql", sub)) => match sub.subcommand() {
            Some(("schema", _)) => {
                println!("{}", kamu_adapter_graphql::schema(catalog).sdl());
                Ok(())
            }
            Some(("query", qsub)) => {
                let result = gql_query(
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
            run_server(
                sub.get_one("address").map(|a| *a),
                sub.get_one("http-port").map(|p| *p),
                catalog,
            )
            .await
        }
        _ => unimplemented!(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn init_logging() {
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_log::LogTracer;
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

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

fn init_dependencies(run_mode: RunMode) -> CatalogBuilder {
    let mut b = dill::CatalogBuilder::new();

    b.add::<infra::QueryServiceImpl>();
    b.bind::<dyn domain::QueryService, infra::QueryServiceImpl>();

    match run_mode {
        RunMode::LocalWorkspace => {
            b.add::<infra::DatasetRepositoryLocalFs>();
            b.bind::<dyn domain::DatasetRepository, infra::DatasetRepositoryLocalFs>();
        }
        RunMode::RemoteS3Url => {
            // Don't register, it is hard to inject arguments, so a manual add is necessary
            // b.add::<infra::DatasetRepositoryS3>();
            // b.bind::<dyn domain::DatasetRepository, infra::DatasetRepositoryS3>();
        }
    }

    b.add::<infra::RemoteRepositoryRegistryImpl>();
    b.bind::<dyn domain::RemoteRepositoryRegistry, infra::RemoteRepositoryRegistryImpl>();

    b.add::<infra::DatasetFactoryImpl>();
    b.bind::<dyn domain::DatasetFactory, infra::DatasetFactoryImpl>();

    b.add::<infra::SyncServiceImpl>();
    b.bind::<dyn domain::SyncService, infra::SyncServiceImpl>();

    b.add::<infra::SearchServiceImpl>();
    b.bind::<dyn domain::SearchService, infra::SearchServiceImpl>();

    // TODO: Externalize configuration
    b.add_value(infra::IpfsGateway {
        url: Url::parse("http://localhost:8080").unwrap(),
        pre_resolve_dnslink: true,
    });
    b.add_value(kamu::infra::utils::ipfs_wrapper::IpfsClient::default());

    b
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn gql_query(query: &str, full: bool, catalog: dill::Catalog) -> String {
    let gql_schema = kamu_adapter_graphql::schema(catalog);
    let response = gql_schema.execute(query).await;

    if full {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        if response.is_ok() {
            serde_json::to_string_pretty(&response.data).unwrap()
        } else {
            for err in &response.errors {
                eprintln!("{}", err)
            }
            // TODO: Error should be propagated as bad exit code
            "".to_owned()
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

// Extractor of dataset identity for smart transfer protocol
#[derive(serde::Deserialize)]
struct DatasetByName {
    dataset_name: opendatafabric::DatasetName,
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn run_server(
    address: Option<std::net::IpAddr>,
    http_port: Option<u16>,
    catalog: dill::Catalog,
) -> Result<(), hyper::Error> {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    let gql_schema = kamu_adapter_graphql::schema(catalog.clone());

    let app = axum::Router::new()
        .route("/", axum::routing::get(root_handler))
        .route(
            "/graphql",
            axum::routing::get(graphql_playground_handler).post(graphql_handler),
        )
        .nest(
            "/:dataset_name",
            kamu_adapter_http::smart_transfer_protocol_routes()
                .layer(kamu_adapter_http::DatasetResolverLayer::new(
                    |axum::extract::Path(p): axum::extract::Path<DatasetByName>| {
                        p.dataset_name.as_local_ref()
                    },
                ))
                .layer(axum::extract::Extension(catalog)),
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(
                    tower_http::cors::CorsLayer::new()
                        .allow_origin(tower_http::cors::Any)
                        .allow_methods(vec![http::Method::GET, http::Method::POST])
                        .allow_headers(tower_http::cors::Any),
                )
                .layer(axum::extract::Extension(gql_schema)),
        );

    let addr = SocketAddr::from((
        address.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
        http_port.unwrap_or(0),
    ));

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    tracing::info!(
        http_endpoint = format!("http://{}", server.local_addr()),
        "Serving traffic"
    );

    server.await
}

/////////////////////////////////////////////////////////////////////////////////////////
// Routes
/////////////////////////////////////////////////////////////////////////////////////////

async fn root_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(
        r#"
        <h1>Kamu API Server</h1>
        <ul>
            <li><a href="/graphql">GraphQL Endpoint</a></li>
            <li><a href="/graphql">GraphQL Playground</a></li>
        </ul>
        "#,
    )
}

async fn graphql_handler(
    schema: axum::extract::Extension<kamu_adapter_graphql::Schema>,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
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
// Workspace Init
/////////////////////////////////////////////////////////////////////////////////////////

fn init_from_local_workspace(workspace_layout: &infra::WorkspaceLayout) {
    if !workspace_layout.root_dir.exists() {
        panic!(
            "Directory is not a kamu workspace: {}",
            workspace_layout.root_dir.display()
        );
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
