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
use ethers::utils::hex::ToHexExt;
use exec::api_client::{OdfApiClient, QueryResponse};
use internal_error::InternalError;
use kamu_oracle_executor as exec;
use serde_json::json;

/////////////////////////////////////////////////////////////////////////////////////////

abigen!(
    Consumer,
    "./abi/Consumer.json",
    event_derives(serde::Deserialize, serde::Serialize),
);

abigen!(
    IOdfAdmin,
    "./abi/IOdfAdmin.json",
    event_derives(serde::Deserialize, serde::Serialize),
);

/////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_e2e() {
    let contracts_dir = std::fs::canonicalize(".")
        .unwrap()
        .join("../../../../kamu-contracts");
    if !contracts_dir.exists() {
        panic!("Contracts dir not found at {}", contracts_dir.display());
    }

    let anvil = ethers::core::utils::Anvil::new().spawn();
    let rpc_endpoint = anvil.endpoint();
    let admin_address = anvil.addresses()[0];
    let admin_key = anvil.keys()[0].to_bytes().encode_hex_with_prefix();
    let executor_address = anvil.addresses()[1];
    let executor_private_key = anvil.keys()[1].to_bytes().encode_hex();
    let oracle_contract_address: Address = "0x5FbDB2315678afecb367f032d93F642f64180aa3"
        .parse()
        .unwrap();
    let consumer_contract_address: Address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
        .parse()
        .unwrap();

    std::process::Command::new("forge")
        .current_dir(&contracts_dir)
        .args(&[
            "script",
            "script/Deploy.s.sol",
            "--fork-url",
            rpc_endpoint.as_str(),
            "--private-key",
            admin_key.as_str(),
            "--broadcast",
        ])
        .status()
        .unwrap()
        .exit_ok()
        .unwrap();

    let config = exec::Config {
        rpc_url: url::Url::parse(&anvil.endpoint()).unwrap(),
        chain_id: anvil.chain_id(),
        oracle_contract_address,
        oracle_contract_first_block: 0,
        executor_address,
        executor_private_key,
        logs_page_size: 1000,
        loop_idle_time_ms: 1000,
        transaction_confirmations: 1,
        transaction_retries: 0,
        api_url: url::Url::parse("http://dontcare.com").unwrap(),
        api_access_token: None,
    };

    let rpc_client = exec::app::init_rpc_client(&config).await.unwrap();
    let api_client = Arc::new(MockOdfApiClient);
    let executor = exec::OdfOracleExecutor::new(config, rpc_client.clone(), api_client);
    let consumer = Consumer::new(consumer_contract_address, rpc_client.clone());
    let oracle_admin = IOdfAdmin::new(oracle_contract_address, rpc_client.clone());

    oracle_admin
        .add_executor(executor_address)
        .from(admin_address)
        .send()
        .await
        .unwrap();

    consumer.start_distribute_rewards().send().await.unwrap();

    executor.run_once(Some(0), None).await.unwrap();

    assert_eq!(consumer.province().call().await.unwrap(), "ON");
    assert_eq!(consumer.total_cases().call().await.unwrap(), 100500);
}

/////////////////////////////////////////////////////////////////////////////////////////

struct MockOdfApiClient;

#[async_trait::async_trait]
impl OdfApiClient for MockOdfApiClient {
    async fn query(&self, _sql: &str) -> Result<QueryResponse, InternalError> {
        Ok(QueryResponse {
            data: vec![vec![json!("ON"), json!(100500)]],
        })
    }
}
