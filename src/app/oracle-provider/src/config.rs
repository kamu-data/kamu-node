// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use alloy::primitives::Address;
use duration_string::DurationString;
use url::Url;

#[derive(confique::Config, Debug)]
pub struct Config {
    /// Interface to listen for HTTP admin traffic on
    #[config(default = "127.0.0.1")]
    pub http_address: String,

    /// Port to listen for HTTP admin traffic on
    #[config(default = 0)]
    pub http_port: u16,

    /// Ethereum-compatible JSON-RPC address
    #[config(default = "http://localhost:8545")]
    pub rpc_url: Url,

    /// ID of the chain used during signing to prevent replay attacks
    #[config(default = 0)]
    pub chain_id: u64,

    /// Address of the oracle contract to read logs from
    pub oracle_contract_address: Address,

    /// Address of this provider's account to use when submitting transactions
    pub provider_address: Address,

    /// Private key of the provider to use when signing transactions.
    pub provider_private_key: String,

    /// Block number to start scanning from on startup (precedence:
    /// scan_from_block, scan_last_blocks, scan_last_blocks_period)
    pub scan_from_block: Option<u64>,

    /// Number of last blocks to scan on startup (precedence: scan_from_block,
    /// scan_last_blocks, scan_last_blocks_period)
    pub scan_last_blocks: Option<u64>,

    /// Time period in which blocks will be scanned on startup (precedence:
    /// scan_from_block, scan_last_blocks, scan_last_blocks_period)
    pub scan_last_blocks_period: Option<DurationString>,

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

    /// Request IDs that provider should skip over (use as a disaster recovery
    /// mechanism only)
    #[config(default = [])]
    pub ignore_requests: Vec<u64>,

    /// Consumer addresses to ignore requests from (use as a disaster recovery
    /// mechanism only)
    #[config(default = [])]
    pub ignore_consumers: Vec<Address>,
}
