// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use alloy::network::EthereumWallet;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer as _;
use internal_error::*;

use crate::api_client::{OdfApiClient, OdfApiClientRest};
use crate::provider::*;
use crate::{Cli, Config};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_RUST_LOG: &str = "debug,kamu=trace,alloy_transport_http=info,alloy_rpc_client=info,\
                                reqwest=info,hyper=info,h2=info";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(args: Cli, config: Config) -> Result<(), InternalError> {
    tracing::info!(?args, ?config, "Starting ODF Oracle provider");

    let http_address = config.http_address.parse().unwrap();
    let http_port = config.http_port;

    let rpc_client = init_rpc_client(&config).await?;
    let api_client = init_api_client(&config).await?;

    let metrics_reg =
        prometheus::Registry::new_custom(Some("kamu_oracle_provider".into()), None).unwrap();
    let metrics =
        OdfOracleProviderMetrics::new(config.chain_id, config.api_url.host_str().unwrap());
    metrics.register(&metrics_reg).int_err()?;

    let provider = OdfOracleProvider::new(config, rpc_client, api_client, metrics);

    let catalog = dill::CatalogBuilder::new().add_value(metrics_reg).build();

    let (http_server, local_addr) = build_http_server(http_address, http_port, catalog).await?;

    tracing::info!("HTTP API is listening on {}", local_addr);

    let shutdown_requested = graceful_shutdown::trap_signals();

    let http_server = Box::pin(async move {
        let server_with_graceful_shutdown = http_server.with_graceful_shutdown(async move {
            shutdown_requested.await;
        });

        server_with_graceful_shutdown.await
    });

    tracing::info!("Entering provider loop");

    tokio::select! {
        res = http_server => { res.int_err() },
        res = provider.run() => { res.int_err() },
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn init_observability() -> observability::init::Guard {
    observability::init::auto(
        observability::config::Config::from_env_with_prefix("KAMU_OTEL_")
            .with_service_name(BINARY_NAME)
            .with_service_version(VERSION)
            .with_default_log_levels(DEFAULT_RUST_LOG),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error("Invalid chain ID. Expected {expected} actual {actual}")]
pub struct InvalidChainId {
    expected: u64,
    actual: u64,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_rpc_client(config: &Config) -> Result<impl Provider + Clone, InternalError> {
    // Prepare wallet
    let signer = PrivateKeySigner::from_str(config.provider_private_key.as_str())
        .unwrap()
        .with_chain_id(Some(config.chain_id));
    let wallet = EthereumWallet::from(signer);

    // Init RPC client
    let rpc_client = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_builtin(config.rpc_url.as_str())
        .await
        .int_err()?;

    let chain_id = rpc_client.get_chain_id().await.int_err()?;
    let last_block = rpc_client.get_block_number().await.int_err()?;
    tracing::info!(chain_id = %chain_id, last_block = %last_block, "Chain info");

    if chain_id != config.chain_id {
        return Err(InvalidChainId {
            expected: config.chain_id,
            actual: chain_id,
        }
        .int_err());
    }

    Ok(rpc_client)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_api_client(config: &Config) -> Result<Arc<dyn OdfApiClient>, InternalError> {
    let client = OdfApiClientRest::new(config.api_url.clone(), config.api_access_token.clone())?;
    Ok(Arc::new(client))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn build_http_server(
    address: std::net::IpAddr,
    http_port: u16,
    catalog: dill::Catalog,
) -> Result<
    (
        axum::serve::Serve<
            tokio::net::TcpListener,
            axum::routing::IntoMakeService<axum::Router>,
            axum::Router,
        >,
        SocketAddr,
    ),
    InternalError,
> {
    let app = axum::Router::new()
        .route(
            "/system/health",
            axum::routing::get(observability::health::health_handler),
        )
        .route(
            "/system/metrics",
            axum::routing::get(observability::metrics::metrics_handler),
        )
        .layer(axum::extract::Extension(catalog));

    let addr = std::net::SocketAddr::from((address, http_port));
    let listener = tokio::net::TcpListener::bind(addr).await.int_err()?;
    let local_addr = listener.local_addr().unwrap();

    let server = axum::serve(listener, app.into_make_service());
    Ok((server, local_addr))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
