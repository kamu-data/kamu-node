// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use ethers::prelude::*;
use url::Url;

#[derive(confique::Config, Debug)]
pub struct Config {
    /// Ethereum-compatible JSON-RPC address
    #[config(default = "http://localhost:8545")]
    pub rpc_url: Url,

    /// Address of the oracle contract to read logs from
    pub oracle_contract_address: Address,

    #[config(default = 0)]
    pub oracle_contract_first_block: u64,

    /// Address of this executor's account to use when submitting transactions
    pub executor_address: Address,

    /// Number of logs to request per one RPC call
    #[config(default = 1000)]
    pub logs_page_size: u64,

    /// Time to sleep while waiting for new blocks
    #[config(default = 1000)]
    pub loop_idle_time_ms: u64,

    /// Number of confirmations to await before considering transaction included
    pub transaction_confirmations: usize,

    /// Number of retries when submitting transaction
    #[config(default = 3)]
    pub transaction_retries: usize,
}
