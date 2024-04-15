// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

/////////////////////////////////////////////////////////////////////////////////////////

use std::net::SocketAddr;

pub(crate) fn build_server(
    address: std::net::IpAddr,
    http_port: Option<u16>,
    catalog: dill::Catalog,
    multi_tenant_workspace: bool,
) -> axum::Server<hyper::server::conn::AddrIncoming, axum::routing::IntoMakeService<axum::Router>> {
    let gql_schema = kamu_adapter_graphql::schema();

    let app = axum::Router::new()
        .route("/", axum::routing::get(root_handler))
        .route(
            "/graphql",
            axum::routing::get(graphql_playground_handler).post(graphql_handler),
        )
        .route(
            "/platform/token/validate",
            axum::routing::get(kamu_adapter_http::platform_token_validate_handler),
        )
        .nest(
            "/odata",
            if multi_tenant_workspace {
                kamu_adapter_odata::router_multi_tenant()
            } else {
                kamu_adapter_odata::router_single_tenant()
            },
        )
        .nest("/", kamu_adapter_http::data::root_router())
        .nest(
            if multi_tenant_workspace {
                "/:account_name/:dataset_name"
            } else {
                "/:dataset_name"
            },
            kamu_adapter_http::add_dataset_resolver_layer(
                axum::Router::new()
                    .nest("/", kamu_adapter_http::smart_transfer_protocol_router())
                    .nest("/", kamu_adapter_http::data::dataset_router()),
                multi_tenant_workspace,
            ),
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
                .layer(axum::extract::Extension(catalog))
                .layer(axum::extract::Extension(gql_schema))
                .layer(kamu_adapter_http::AuthenticationLayer::new()),
        );

    let addr = SocketAddr::from((address, http_port.unwrap_or(0)));

    axum::Server::bind(&addr).serve(app.into_make_service())
}

/////////////////////////////////////////////////////////////////////////////////////////
// Routes
/////////////////////////////////////////////////////////////////////////////////////////

async fn root_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(
        r#"
        <h1>Kamu API Server</h1>
        <ul>
            <li><a href="/graphql">GraphQL Playground</a></li>
        </ul>
        "#,
    )
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn graphql_handler(
    axum::extract::Extension(schema): axum::extract::Extension<kamu_adapter_graphql::Schema>,
    axum::extract::Extension(catalog): axum::extract::Extension<dill::Catalog>,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let graphql_request = req.into_inner().data(catalog);
    schema.execute(graphql_request).await.into()
}

/////////////////////////////////////////////////////////////////////////////////////////

async fn graphql_playground_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

/////////////////////////////////////////////////////////////////////////////////////////
