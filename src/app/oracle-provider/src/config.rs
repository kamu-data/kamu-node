// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use alloy::primitives::Address;
use url::Url;

#[derive(confique::Config, Debug)]
pub struct Config {
    /// Ethereum-compatible JSON-RPC address
    #[config(default = "http://localhost:8545")]
    pub rpc_url: Url,

    /// ID of the chain used during signing to prevent replay attacks
    #[config(default = 0)]
    pub chain_id: u64,

    /// Address of the oracle contract to read logs from
    pub oracle_contract_address: Address,

    #[config(default = 0)]
    pub oracle_contract_first_block: u64,

    /// Address of this provider's account to use when submitting transactions
    pub provider_address: Address,

    /// Private key of the provider to use when signing transactions.
    pub provider_private_key: String,

    /// Number of blocks to examine per one getLogs RPC request when catching up
    #[config(default = 100_000)]
    pub blocks_stride: u64,

    /// Time to sleep while waiting for new blocks
    #[config(default = 1000)]
    pub loop_idle_time_ms: u64,

    /// Number of confirmations to await before considering transaction included
    pub transaction_confirmations: u64,

    /// Timeout when submitting a transaction
    #[config(default = 60)]
    pub transaction_timeout_s: u64,

    /// URL of the ODF-compatible API server that will execute requests
    #[config(default = "http://localhost:8080")]
    pub api_url: Url,

    /// API token to use for authentication with the server
    pub api_access_token: Option<String>,
}
