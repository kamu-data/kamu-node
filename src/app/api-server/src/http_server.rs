// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::net::SocketAddr;

use database_common_macros::transactional_handler;
use http_common::ApiError;
use indoc::indoc;
use internal_error::{InternalError, ResultIntoInternal};
use kamu::domain::TenancyConfig;
use utoipa::OpenApi as _;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn build_server(
    address: std::net::IpAddr,
    http_port: Option<u16>,
    catalog: dill::Catalog,
    tenancy_config: TenancyConfig,
) -> Result<
    (
        axum::serve::Serve<axum::routing::IntoMakeService<axum::Router>, axum::Router>,
        SocketAddr,
    ),
    InternalError,
> {
    let gql_schema = kamu_adapter_graphql::schema();

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .route("/", axum::routing::get(root_handler))
        .route(
            "/graphql",
            axum::routing::get(graphql_playground_handler).post(graphql_handler),
        )
        .routes(routes!(kamu_adapter_http::platform_login_handler))
        .routes(routes!(kamu_adapter_http::platform_token_validate_handler))
        .routes(routes!(
            kamu_adapter_http::platform_file_upload_prepare_post_handler
        ))
        .routes(routes!(
            kamu_adapter_http::platform_file_upload_post_handler,
            kamu_adapter_http::platform_file_upload_get_handler
        ))
        .merge(kamu_adapter_http::data::root_router())
        .merge(kamu_adapter_http::general::root_router())
        .nest(
            "/odata",
            match tenancy_config {
                TenancyConfig::MultiTenant => kamu_adapter_odata::router_multi_tenant(),
                TenancyConfig::SingleTenant => kamu_adapter_odata::router_single_tenant(),
            },
        )
        .nest(
            match tenancy_config {
                TenancyConfig::MultiTenant => "/:account_name/:dataset_name",
                TenancyConfig::SingleTenant => "/:dataset_name",
            },
            kamu_adapter_http::add_dataset_resolver_layer(
                OpenApiRouter::new()
                    .merge(kamu_adapter_http::smart_transfer_protocol_router())
                    .merge(kamu_adapter_http::data::dataset_router()),
                tenancy_config,
            ),
        )
        .layer(kamu_adapter_http::AuthenticationLayer::new())
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(vec![http::Method::GET, http::Method::POST])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(observability::axum::http_layer())
        // Note: Healthcheck and metrics routes are placed before the tracing layer (layers
        // execute bottom-up) to avoid spam in logs
        .route(
            "/system/health",
            axum::routing::get(observability::health::health_handler),
        )
        .route(
            "/system/metrics",
            axum::routing::get(observability::metrics::metrics_handler),
        )
        .layer(axum::extract::Extension(gql_schema))
        .layer(axum::extract::Extension(catalog))
        .split_for_parts();

    let router =
        router.merge(utoipa_swagger_ui::SwaggerUi::new("/swagger").url("/openapi.json", api));

    let addr = SocketAddr::from((address, http_port.unwrap_or(0)));
    let listener = tokio::net::TcpListener::bind(addr).await.int_err()?;
    let local_addr = listener.local_addr().unwrap();

    let server = axum::serve(listener, router.into_make_service());
    Ok((server, local_addr))
}

/////////////////////////////////////////////////////////////////////////////////////////
// Routes
/////////////////////////////////////////////////////////////////////////////////////////

async fn root_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(indoc!(
        r#"
        <h1>Kamu API Server</h1>
        <ul>
            <li><a href="/graphql">GraphQL Playground</li>
            <li><a href="/swagger/">Swagger UI</li>
        </ul>
        "#
    ))
}

/////////////////////////////////////////////////////////////////////////////////////////

#[transactional_handler]
async fn graphql_handler(
    axum::extract::Extension(schema): axum::extract::Extension<kamu_adapter_graphql::Schema>,
    axum::extract::Extension(catalog): axum::extract::Extension<dill::Catalog>,
    req: async_graphql_axum::GraphQLRequest,
) -> Result<async_graphql_axum::GraphQLResponse, ApiError> {
    let graphql_request = req.into_inner().data(catalog);
    let graphql_response = schema.execute(graphql_request).await.into();

    Ok(graphql_response)
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn graphql_playground_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// OpenAPI root
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(utoipa::OpenApi)]
#[openapi(
    modifiers(&TokenAuthAddon),
    tags(
        (name = "odf-core", description = "Core ODF APIs"),
        (name = "odf-transfer", description = "ODF Data Transfer APIs"),
        (name = "odf-query", description = "ODF Data Query APIs"),
        (name = "kamu", description = "General Node APIs"),
        (name = "kamu-odata", description = "OData Adapter"),
    )
)]
struct ApiDoc;

struct TokenAuthAddon;

impl utoipa::Modify for TokenAuthAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::*;

        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("AccessToken")
                        .build(),
                ),
            );
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
