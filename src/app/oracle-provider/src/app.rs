// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use ethers::prelude::*;
use internal_error::*;

use crate::api_client::{OdfApiClient, OdfApiClientRest};
use crate::provider::*;
use crate::{Cli, Config};

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(args: Cli, config: Config) -> Result<(), InternalError> {
    tracing::info!(?args, ?config, "Starting ODF Oracle provider");

    let rpc_client = init_rpc_client(&config).await?;
    let api_client = init_api_client(&config).await?;
    let provider = OdfOracleProvider::new(config, rpc_client, api_client);

    tracing::info!("Entering provider loop");
    provider.run().await?;

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error("Invalid chain ID. Expected {expected} actual {actual}")]
pub struct InvalidChainId {
    expected: u64,
    actual: u64,
}

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_rpc_client(config: &Config) -> Result<Arc<impl Middleware>, InternalError> {
    // Prepare wallet
    let wallet = LocalWallet::try_from(config.provider_private_key.as_str())
        .unwrap()
        .with_chain_id(config.chain_id);

    // Init RPC client
    let provider = Provider::<Http>::connect(config.rpc_url.as_str()).await;
    let rpc_client = ethers::middleware::SignerMiddleware::new(provider, wallet);

    let chain_id = rpc_client.get_chainid().await.int_err()?.as_u64();
    let last_block = rpc_client.get_block_number().await.int_err()?;
    tracing::info!(chain_id = %chain_id, last_block = %last_block, "Chain info");

    if chain_id != config.chain_id {
        return Err(InvalidChainId {
            expected: config.chain_id,
            actual: chain_id,
        }
        .int_err());
    }

    Ok(Arc::new(rpc_client))
}

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn init_api_client(config: &Config) -> Result<Arc<dyn OdfApiClient>, InternalError> {
    let client = OdfApiClientRest::new(config.api_url.clone(), config.api_access_token.clone())?;
    Ok(Arc::new(client))
}

/////////////////////////////////////////////////////////////////////////////////////////
