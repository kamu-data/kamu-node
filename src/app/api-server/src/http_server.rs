// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use database_common_macros::transactional_handler;
use http_common::ApiError;
use internal_error::{InternalError, ResultIntoInternal};
use kamu::domain::TenancyConfig;
use kamu_adapter_http::DatasetAuthorizationLayer;
use observability::axum::{panic_handler, unknown_fallback_handler};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tower_http::catch_panic::CatchPanicLayer;
use utoipa_axum::router::OpenApiRouter;

use crate::ui_configuration::UIConfiguration;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn build_server(
    address: std::net::IpAddr,
    http_port: Option<u16>,
    catalog: dill::Catalog,
    tenancy_config: TenancyConfig,
    ui_config: UIConfiguration,
    e2e_http_port: Option<u16>,
    e2e_output_data_path: Option<&PathBuf>,
) -> Result<
    (
        axum::serve::Serve<
            tokio::net::TcpListener,
            axum::routing::IntoMakeService<axum::Router>,
            axum::Router,
        >,
        SocketAddr,
        Option<Arc<Notify>>,
    ),
    InternalError,
> {
    let gql_schema = kamu_adapter_graphql::schema();

    let addr = SocketAddr::from((address, e2e_http_port.or(http_port).unwrap_or(0)));
    let listener = TcpListener::bind(addr).await.int_err()?;
    let local_addr = listener.local_addr().unwrap();

    let mut open_api_router = OpenApiRouter::with_openapi(
        kamu_adapter_http::openapi::spec_builder(
            crate::app::VERSION,
            indoc::indoc!(
                r#"
                [Kamu Node](https://docs.kamu.dev/node/) is a set of Kubernetes-native applications that can be deployed in any cloud or on-prem to:

                - Operate the stream processing pipelines for a certain set of data flows
                - Continuously ingest external data into signed root datasets
                - Continuously verify a configured set of pipelines to catch malicious behavior
                - Execute queries on co-located data and provide data via variety of protocols
                - Provide data to smart contracts via multi-chain oracle mechanism

                Nodes are the building pieces of the [Open Data Fabric](https://docs.kamu.dev/odf/) network and the primary way of contributing resources to the network. Unlike blockchain nodes that maintain a single ledger, Kamu nodes can form loosely connected clusters based on vested interests of their operators in certain data pipelines.

                ## Auth
                Some operation require an **API token**. If you are running a Kamu Web Platform instance alongside this node you can obtain the Personal Access Token by logging in, going to account settings, and into the "Access Tokens" tab.

                ## Resources
                - [Documentation](https://docs.kamu.dev)
                - [Discord](https://discord.gg/nU6TXRQNXC)
                - [Other protocols](https://docs.kamu.dev/node/protocols/)
                - [Open Data Fabric specification](https://docs.kamu.dev/odf/)
                "#
            ),
        )
        .build(),
    )
    .route(
        "/graphql",
        axum::routing::post(graphql_handler),
    )
    .merge(server_console::router("Kamu API Server".to_string(), format!("v{}", crate::app::VERSION)).into())
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
            TenancyConfig::MultiTenant => "/{account_name}/{dataset_name}",
            TenancyConfig::SingleTenant => "/{dataset_name}",
        },
        kamu_adapter_http::add_dataset_resolver_layer(
            OpenApiRouter::new()
                .merge(kamu_adapter_http::smart_transfer_protocol_router())
                .merge(kamu_adapter_http::data::dataset_router())
                .layer(DatasetAuthorizationLayer::default()),
            tenancy_config,
        ),
    );

    if !ui_config.feature_flags.allow_anonymous {
        open_api_router = open_api_router.layer(kamu_adapter_http::AuthPolicyLayer::new());
    }

    let (mut router, api) = open_api_router
        .nest(
            "/platform",
            kamu_adapter_http::platform::root_router(ui_config.feature_flags.allow_anonymous),
        )
        .route("/ui-config", axum::routing::get(ui_configuration_handler))
        .layer(kamu_adapter_http::AuthenticationLayer::new())
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(vec![http::Method::GET, http::Method::POST])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(observability::axum::http_layer())
        .layer(CatchPanicLayer::custom(panic_handler))
        // Note: Healthcheck, metrics, and OpenAPI routes are placed before the tracing layer
        // (layers execute bottom-up) to avoid spam in logs
        .route(
            "/system/health",
            axum::routing::get(observability::health::health_handler),
        )
        .route(
            "/system/metrics",
            axum::routing::get(observability::metrics::metrics_handler),
        )
        .route(
            "/system/info",
            axum::routing::get(observability::build_info::build_info_handler),
        )
        .merge(kamu_adapter_http::openapi::router().into())
        .fallback(unknown_fallback_handler)
        .layer(axum::extract::Extension(gql_schema))
        .layer(axum::extract::Extension(catalog))
        .layer(axum::extract::Extension(ui_config))
        .split_for_parts();

    let maybe_shutdown_notify = if e2e_output_data_path.is_some() {
        let shutdown_notify = Arc::new(Notify::new());

        router = router.nest(
            "/e2e",
            kamu_adapter_http::e2e::e2e_router(shutdown_notify.clone()),
        );

        Some(shutdown_notify)
    } else {
        None
    };

    let router = router.layer(axum::extract::Extension(std::sync::Arc::new(api)));

    let server = axum::serve(listener, router.into_make_service());
    Ok((server, local_addr, maybe_shutdown_notify))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Routes
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn ui_configuration_handler(
    axum::extract::Extension(ui_config): axum::extract::Extension<UIConfiguration>,
) -> axum::Json<UIConfiguration> {
    axum::Json(ui_config)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
