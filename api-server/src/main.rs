mod cli_parser;

use std::path::{Path, PathBuf};

use async_graphql_warp::GraphQLResponse;
use clap::value_t_or_exit;
use url::Url;

/////////////////////////////////////////////////////////////////////////////////////////

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LOGGING_CONFIG: &str = "info";

/////////////////////////////////////////////////////////////////////////////////////////

async fn index_route() -> &'static str {
    r#"
    <h1>Kamu API Server</h1>
    <ul>
        <li><a href="/graphql">GraphQL Endpoint</a></li>
        <li><a href="/playground">GraphQL Playground</a></li>
    </ul>
    "#
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn graphql_route(
    (schema, request): (kamu_api_server::gql::Schema, async_graphql::Request),
) -> Result<GraphQLResponse, std::convert::Infallible> {
    Ok(GraphQLResponse::from(schema.execute(request).await))
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn playground_route() -> warp::http::Result<warp::http::Response<String>> {
    use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

    warp::http::Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        ))
}

/////////////////////////////////////////////////////////////////////////////////////////

fn init_logging() {
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_log::LogTracer;
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

    // Redirect all standard logging to tracing events
    LogTracer::init().expect("Failed to set LogTracer");

    // Use configuration from RUST_LOG env var if provided
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(DEFAULT_LOGGING_CONFIG.to_owned()));

    // TODO: Use non-blocking writer?
    // Configure Bunyan JSON formatter
    let formatting_layer = BunyanFormattingLayer::new(BINARY_NAME.to_owned(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn gql_query(query: &str, full: bool, catalog: dill::Catalog) -> String {
    let gql_schema = kamu_api_server::gql::schema(catalog);
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

async fn run_server(
    address: std::net::IpAddr,
    http_port: u16,
    catalog: dill::Catalog,
) -> std::io::Result<()> {
    use warp::Filter;

    tracing::info!(
        "HTTP server: \n\
         - Server root: http://{addr}:{port}\n\
         - GraphQL playground: http://{addr}:{port}/playground",
        addr = address,
        port = http_port
    );

    let index = warp::path::end().and(warp::get()).then(index_route);

    let graphql = warp::path("graphql")
        .and(warp::path::end())
        .and(async_graphql_warp::graphql(kamu_api_server::gql::schema(
            catalog.clone(),
        )))
        .and_then(graphql_route);

    let playground = warp::path("playground")
        .and(warp::path::end())
        .and(warp::get())
        .then(playground_route);

    let routes = index
        .or(graphql)
        .or(playground)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(["GET", "POST"])
                .allow_headers(["content-type"]),
        )
        .with(warp::trace::request());

    warp::serve(routes).run((address, http_port)).await;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let mut catalog = dill::Catalog::new();

    catalog.add::<kamu::infra::QueryServiceImpl>();
    catalog
        .bind::<dyn kamu::domain::QueryService, kamu::infra::QueryServiceImpl>()
        .unwrap();

    let matches = cli_parser::cli(BINARY_NAME, VERSION).get_matches();

    let cwd = Path::new(".").canonicalize().unwrap();
    let metadata_repo_url = Url::parse(
        matches
            .value_of("metadata-repo")
            .unwrap_or(Url::from_file_path(&cwd).unwrap().as_str()),
    )
    .unwrap();

    match metadata_repo_url.scheme() {
        "file" => init_metadata_repo_from_local_workspace(
            &metadata_repo_url.to_file_path().unwrap(),
            &mut catalog,
        ),
        _ => init_metadata_repo_from_synced_repo(metadata_repo_url, &mut catalog),
    }

    match matches.subcommand() {
        ("gql", Some(sub)) => match sub.subcommand() {
            ("schema", _) => {
                println!("{}", kamu_api_server::gql::schema(catalog).sdl());
                Ok(())
            }
            ("query", Some(qsub)) => {
                let result = gql_query(
                    qsub.value_of("query").unwrap(),
                    qsub.is_present("full"),
                    catalog,
                )
                .await;
                print!("{}", result);
                Ok(())
            }
            _ => unimplemented!(),
        },
        ("run", Some(sub)) => {
            run_server(
                sub.value_of("address").unwrap().parse().unwrap(),
                value_t_or_exit!(sub.value_of("http-port"), u16),
                catalog,
            )
            .await
        }
        _ => unimplemented!(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Workspace Init
/////////////////////////////////////////////////////////////////////////////////////////

fn init_metadata_repo_from_local_workspace(workspace_root_dir: &Path, catalog: &mut dill::Catalog) {
    let workspace_layout = kamu::infra::WorkspaceLayout::new(&workspace_root_dir);
    if !workspace_layout.kamu_root_dir.exists() {
        panic!(
            "Directory is not a kamu workspace: {}",
            workspace_root_dir.display()
        );
    }
    let volume_layout = kamu::infra::VolumeLayout::new(&workspace_layout.local_volume_dir);

    catalog.add::<kamu::infra::MetadataRepositoryImpl>();
    catalog
        .bind::<dyn kamu::domain::MetadataRepository, kamu::infra::MetadataRepositoryImpl>()
        .unwrap();

    catalog.add_value(workspace_layout);
    catalog.add_value(volume_layout);
}

/////////////////////////////////////////////////////////////////////////////////////////

fn init_metadata_repo_from_synced_repo(repo_url: Url, catalog: &mut dill::Catalog) {
    let workspace_root_dir = PathBuf::from(
        std::env::var("KAMU_SYNC_DIR")
            .ok()
            .expect("Please specify the directory where to store local copy of the repository using KAMU_SYNC_DIR env var"),
    );

    let workspace_layout = kamu::infra::WorkspaceLayout::new(&workspace_root_dir);
    if !workspace_layout.kamu_root_dir.exists() {
        tracing::info!(
            message = "Creating local workspace as sync target from repository",
            ?workspace_root_dir,
            repo_url = repo_url.to_string().as_str(),
        );
        kamu::infra::WorkspaceLayout::create(&workspace_root_dir).unwrap();
    } else {
        tracing::info!(
            message = "Using existing workspace as sync target from repository",
            ?workspace_root_dir,
            repo_url = repo_url.to_string().as_str(),
        );
    }

    init_metadata_repo_from_local_workspace(&workspace_root_dir, catalog);
    catalog.add::<kamu::infra::RepositoryFactory>();
    catalog.add::<kamu::infra::SyncServiceImpl>();
    catalog
        .bind::<dyn kamu::domain::SyncService, kamu::infra::SyncServiceImpl>()
        .unwrap();

    let metadata_repo = catalog
        .get_one::<dyn kamu::domain::MetadataRepository>()
        .unwrap();

    let repo_id = opendatafabric::RepositoryID::new_unchecked("remote");

    let _ = metadata_repo.delete_repository(repo_id);
    metadata_repo.add_repository(repo_id, repo_url).unwrap();

    // Run first iteration synchronously to catch any misconfiguration
    let start = chrono::Utc::now();
    loop {
        match kamu_api_server::repo_sync::sync_all_from_repo(
            catalog.get_one().unwrap(),
            catalog.get_one().unwrap(),
            catalog.get_one().unwrap(),
            repo_id,
        ) {
            Ok(_) => break,
            Err(
                kamu::domain::SyncError::IOError(_) | kamu::domain::SyncError::ProtocolError(_),
            ) => {
                tracing::warn!("Failed to do initial sync from repo, waiting a bit longer");
                ()
            }
            e @ _ => e.expect("Terminating on sync failure"),
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        if chrono::Utc::now() - start > chrono::Duration::seconds(10) {
            break;
        }
    }

    let catalog = catalog.clone();
    std::thread::spawn(move || kamu_api_server::repo_sync::repo_sync_loop(catalog, repo_id));
}
