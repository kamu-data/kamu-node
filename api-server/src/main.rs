mod cli_parser;

use std::path::Path;

use actix_web as ws;
use async_graphql_actix_web as ws_gql;
use clap::value_t_or_exit;
use url::Url;

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LOGGING_CONFIG: &str = "info";

async fn index_route() -> impl ws::Responder {
    format!("Hello!")
}

async fn graphql_route(
    req: ws_gql::Request,
    schema: ws::web::Data<kamu_api_server::gql::Schema>,
) -> ws_gql::Response {
    schema.execute(req.into_inner()).await.into()
}

async fn playground_route() -> ws::Result<ws::HttpResponse> {
    use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

    Ok(ws::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}

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

async fn run_server(address: &str, http_port: u16, catalog: dill::Catalog) -> std::io::Result<()> {
    use actix_web::{http, middleware, web, App, HttpServer};

    tracing::info!(
        "HTTP server: \n\
         - Server root: http://{addr}:{port}\n\
         - GraphQL playground: http://{addr}:{port}/playground",
        addr = address,
        port = http_port
    );

    HttpServer::new(move || {
        App::new()
            .data(kamu_api_server::gql::schema(catalog.clone()))
            .wrap(middleware::Compress::default())
            .wrap(
                actix_cors::Cors::default()
                    // TODO: Security
                    .allow_any_origin()
                    //.allowed_origin("http://127.0.0.1:8080")
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(tracing_actix_web::TracingLogger)
            .service(web::resource("/").route(web::get().to(index_route)))
            .service(web::resource("/graphql").route(web::post().to(graphql_route)))
            .service(web::resource("/playground").route(web::get().to(playground_route)))
    })
    .bind((address, http_port))?
    .workers(1)
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let mut catalog = dill::Catalog::new();
    catalog.add::<kamu::infra::MetadataRepositoryImpl>();
    catalog
        .bind::<dyn kamu::domain::MetadataRepository, kamu::infra::MetadataRepositoryImpl>()
        .unwrap();

    let matches = cli_parser::cli(BINARY_NAME, VERSION).get_matches();

    let cwd = Path::new(".").canonicalize().unwrap();
    let metadata_repo_url = Url::parse(
        matches
            .value_of("metadata-repo")
            .unwrap_or(Url::from_file_path(&cwd).unwrap().as_str()),
    )
    .unwrap();

    let workspace_layout = match metadata_repo_url.scheme() {
        "file" => {
            let workspace_root_dir = metadata_repo_url.to_file_path().unwrap();
            kamu::infra::WorkspaceLayout::new(&workspace_root_dir)
        }
        _ => panic!("Unsupported metadata repo URL scheme"),
    };

    catalog.add_value(workspace_layout);

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
                sub.value_of("address").unwrap(),
                value_t_or_exit!(sub.value_of("http-port"), u16),
                catalog,
            )
            .await
        }
        _ => unimplemented!(),
    }
}
