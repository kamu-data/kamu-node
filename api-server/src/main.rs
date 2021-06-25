use actix_web as ws;
use async_graphql_actix_web as ws_gql;

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
//const VERSION: &str = env!("CARGO_PKG_VERSION");
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

pub fn init_logging() {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{http, middleware, web, App, HttpServer};

    init_logging();

    tracing::info!("Starting the web server");

    let schema = kamu_api_server::gql::schema();
    println!("{}", schema.sdl());

    HttpServer::new(move || {
        App::new()
            .data(kamu_api_server::gql::schema())
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
    .bind("127.0.0.1:8080")?
    .workers(1)
    .run()
    .await
}
