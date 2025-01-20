// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;
use std::sync::Arc;

use alloy::primitives::Address;
use alloy::sol;
use kamu_oracle_provider::api_client::*;
use kamu_oracle_provider::{self as provider};
use serde_json::json;

/////////////////////////////////////////////////////////////////////////////////////////

sol! {
    #[sol(rpc)]
    contract Consumer {
        string public province;
        uint64 public totalCases;

        constructor(address oracleAddr);
        function initiateQuery() public;
    }
}

sol! {
    #[sol(rpc)]
    interface IOdfAdmin {
        // Emitted when a provider is authorized
        event AddProvider(address indexed providerAddr);

        // Emitted when a provider authorization is revoked
        event RemoveProvider(address indexed providerAddr);

        // Authorizes a provider to supply results
        function addProvider(address providerAddr) external;

        // Revoke the authorization from a provider to supply results
        function removeProvider(address providerAddr) external;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn get_contracts_dir() -> PathBuf {
    let dir = if let Ok(dir) = std::env::var("KAMU_CONTRACTS_DIR") {
        PathBuf::from(dir)
    } else {
        std::fs::canonicalize(".")
            .unwrap()
            .join("../../../../kamu-contracts")
    };
    if !dir.exists() {
        panic!("Contracts dir not found at {}", dir.display());
    }
    dir
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test_group::group(e2e, oracle, flaky)]
#[test_log::test(tokio::test)]
async fn test_oracle_e2e() {
    let contracts_dir = get_contracts_dir();

    let anvil = alloy::node_bindings::Anvil::new().spawn();
    let rpc_endpoint = anvil.endpoint();
    let admin_address = anvil.addresses()[0];
    let admin_key = hex::encode(anvil.keys()[0].to_bytes());
    let provider_address = anvil.addresses()[1];
    let provider_private_key = hex::encode(anvil.keys()[1].to_bytes());
    let oracle_contract_address: Address = "0x5FbDB2315678afecb367f032d93F642f64180aa3"
        .parse()
        .unwrap();
    let consumer_contract_address: Address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
        .parse()
        .unwrap();

    std::process::Command::new("forge")
        .current_dir(&contracts_dir)
        .args([
            "script",
            "script/Deploy.s.sol",
            "--fork-url",
            rpc_endpoint.as_str(),
            "--private-key",
            admin_key.as_str(),
            "--broadcast",
        ])
        .status()
        .expect("Failed to deploy contracts. Is foundry installed?")
        .exit_ok()
        .unwrap();

    let config = provider::Config {
        http_address: "127.0.0.1".into(),
        http_port: 0,
        rpc_url: url::Url::parse(&anvil.endpoint()).unwrap(),
        chain_id: anvil.chain_id(),
        oracle_contract_address,
        scan_from_block: Some(0),
        scan_last_blocks: None,
        scan_last_blocks_period: None,
        provider_address,
        provider_private_key,
        blocks_stride: 100_000,
        loop_idle_time_ms: 1000,
        transaction_confirmations: 1,
        transaction_timeout_s: 5,
        api_url: url::Url::parse("http://dontcare.com").unwrap(),
        api_access_token: None,
        ignore_requests: Vec::new(),
        ignore_consumers: Vec::new(),
    };

    // Authorize provider and generate a request
    let admin_rpc_client = alloy::providers::ProviderBuilder::new()
        .with_recommended_fillers()
        .on_builtin(&rpc_endpoint)
        .await
        .unwrap();

    let oracle_admin = IOdfAdmin::new(oracle_contract_address, admin_rpc_client.clone());
    let consumer = Consumer::new(consumer_contract_address, admin_rpc_client.clone());

    oracle_admin
        .addProvider(provider_address)
        .from(admin_address)
        .send()
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    consumer
        .initiateQuery()
        .from(admin_address)
        .send()
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    // Setup and run provider
    let rpc_client = provider::app::init_rpc_client(&config).await.unwrap();

    let api_client = Arc::new(MockOdfApiClient);

    // let api_client = Arc::new(
    //     OdfApiClientRest::new(url::Url::parse("https://api.demo.kamu.dev").unwrap(), None).unwrap(),
    // );

    let provider = provider::OdfOracleProvider::new(
        config,
        rpc_client.clone(),
        api_client,
        provider::OdfOracleProviderMetrics::new(0, "localhost"),
    );

    provider.run_once(Some(0), None).await.unwrap();

    assert_eq!(consumer.province().call().await.unwrap().province, "ON");
    assert_eq!(
        consumer.totalCases().call().await.unwrap().totalCases,
        100500
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Replace with real API server
struct MockOdfApiClient;

#[async_trait::async_trait]
impl OdfApiClient for MockOdfApiClient {
    async fn query(&self, request: QueryRequest) -> Result<QueryResponse, QueryError> {
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "include": ["Input"],
                "query": request.query,
                "queryDialect": "SqlDataFusion",
                "dataFormat": "JsonAoa",
                "datasets": [{
                    "alias": "kamu/covid19.canada.case-details",
                    "id": "did:odf:fed01dcda047d51fc88246c730db522d36791c9e2286af23d9f2b920f09c65952e3d0",
                }],
            }),
        );

        Ok(QueryResponse {
            input: Some(QueryRequest {
                include: vec![Include::Input],
                query: request.query,
                query_dialect: Some(QueryDialect::SqlDataFusion),
                data_format: Some(DataFormat::JsonAoa),
                datasets: Some(vec![DatasetState {
                    id: odf::DatasetID::from_did_str("did:odf:fed01dcda047d51fc88246c730db522d36791c9e2286af23d9f2b920f09c65952e3d0").unwrap(),
                    alias: "kamu/covid19.canada.case-details".to_string(),
                    block_hash: Some(odf::Multihash::from_multibase("f162080b0979126041b122a0b0851f286503e8a501b03ba2008bf260b348801abc76f").unwrap()),
                }]),
                skip: Some(0),
                limit: Some(1000),
            }),
            output: Outputs{
                data: json!([["ON", 100500]]),
                data_format: DataFormat::JsonAoa,
            }
        })
    }
}
