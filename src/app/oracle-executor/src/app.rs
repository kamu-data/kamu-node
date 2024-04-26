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
use url::Url;

use crate::{Cli, Config};

/////////////////////////////////////////////////////////////////////////////////////////

abigen!(
    IOdfProvider,
    "./abi/IOdfProvider.json",
    event_derives(serde::Deserialize, serde::Serialize),
);

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct OdfRequest {
    pub id: u64,
    pub data: OdfRequestData,
    pub meta: ethers::contract::LogMeta,
}

#[derive(Debug, serde::Deserialize)]
struct OdfRequestData {
    pub sql: String,
}

#[derive(Debug, serde::Serialize)]
struct OdfResult {
    pub data: Vec<Vec<ciborium::Value>>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Deserialize)]
struct ApiQueryResponse {
    data: Vec<serde_json::Value>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error("Executor is not authorized to provide results to the oracle contract")]
pub struct ExecutorUnauthorized;

/////////////////////////////////////////////////////////////////////////////////////////

struct OdfOracleExecutor<P: JsonRpcClient> {
    config: Config,
    rpc_client: Arc<Provider<P>>,
    oracle_contract: IOdfProvider<Provider<P>>,
    api_client: reqwest::Client,
    api_query_url: Url,
}

impl<P: JsonRpcClient + 'static> OdfOracleExecutor<P> {
    fn new(config: Config, rpc_client: Arc<Provider<P>>, api_client: reqwest::Client) -> Self {
        let oracle_contract = IOdfProvider::new(config.oracle_contract_address, rpc_client.clone());

        let api_query_url = Url::parse(&format!(
            "{}/query",
            config.api_url.as_str().trim_end_matches('/')
        ))
        .unwrap();

        Self {
            config,
            rpc_client,
            api_client,
            oracle_contract,
            api_query_url,
        }
    }

    async fn check_authorized(&self) -> Result<(), InternalError> {
        // This expects that the first the first thing the contract checks is whether
        // the executor is authorized to submit results.
        match self
            .oracle_contract
            .can_provide_results(self.config.executor_address)
            .from(self.config.executor_address)
            .call()
            .await
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(ExecutorUnauthorized.int_err()),
            Err(err) => Err(err.int_err()),
        }
    }

    async fn run(self) -> Result<(), InternalError> {
        let mut from_block = ethers::types::U64::from(self.config.oracle_contract_first_block);
        let mut idle_start = None;

        loop {
            // TODO: Operate on blocks that have >N confirmations to avoid running into too
            // many reorgs?
            let last_block = self.rpc_client.get_block_number().await.int_err()?;

            // TODO: Reorg resistance
            if from_block > last_block {
                if idle_start.is_none() {
                    tracing::debug!("Waiting for new blocks");
                    idle_start = Some(std::time::Instant::now());
                }
                tokio::time::sleep(std::time::Duration::from_millis(
                    self.config.loop_idle_time_ms,
                ))
                .await;
                continue;
            } else {
                idle_start = None;
            }

            let pending_requests = self.scan_block_range(from_block, last_block).await?;

            // TODO: Handle malformed requests
            let requests = pending_requests
                .into_iter()
                .map(|(event, meta)| Self::decode_request(event, meta))
                .collect::<Result<Vec<_>, _>>()?;

            // TODO: Batch responses into single transaction
            for request in requests {
                tracing::info!(?request, "Processing request");

                let result = self.execute_query(&request).await?;

                tracing::info!(request_id = ?request.id, ?result, "Sending result");
                self.send_result(request.id, result).await?;
            }

            // TODO: Chain reorg resistance
            from_block = last_block + 1;
        }
    }

    // TODO: This code is much more complex than it should be because of the issue
    // in ethers where calling `contract.events()` sets the address filter but
    // not the topic0 filter, thus when used on an interface rather than final
    // contract - there might be other events present in result and decoding
    // will fail.
    // See: https://github.com/gakonst/ethers-rs/issues/2497
    #[tracing::instrument(level = "info", skip(self))]
    async fn scan_block_range(
        &self,
        from_block: ethers::types::U64,
        to_block: ethers::types::U64,
    ) -> Result<Vec<(i_odf_provider::SendRequestFilter, LogMeta)>, InternalError> {
        let mut pending_requests = std::collections::HashMap::new();

        let filter = Filter::new()
            .address(self.config.oracle_contract_address)
            .topic0(vec![
                i_odf_provider::SendRequestFilter::signature(),
                i_odf_provider::ProvideResultFilter::signature(),
            ])
            .from_block(from_block)
            .to_block(to_block);

        let mut log_stream = self
            .rpc_client
            .get_logs_paginated(&filter, self.config.logs_page_size);

        while let Some(log) = log_stream.next().await.transpose().int_err()? {
            assert!(!log.removed.unwrap_or_default());

            let meta = LogMeta::from(&log);
            let event = IOdfProviderEvents::decode_log(&abi::RawLog {
                topics: log.topics,
                data: log.data.to_vec(),
            })
            .int_err()?;

            match event {
                IOdfProviderEvents::SendRequestFilter(event) => {
                    tracing::debug!(request_id = ?event.request_id, "Adding pending request");
                    pending_requests.insert(event.request_id, (event, meta));
                }
                IOdfProviderEvents::ProvideResultFilter(event) => {
                    tracing::debug!(request_id = ?event.request_id, "Removing request as fulfilled");
                    pending_requests.remove(&event.request_id);
                }
            }
        }

        let pending_requests: Vec<_> = pending_requests.into_values().collect();
        if pending_requests.is_empty() {
            tracing::debug!("No pending requests");
        } else {
            tracing::debug!(?pending_requests, "Pending requests");
        }

        Ok(pending_requests)
    }

    fn decode_request(
        event: i_odf_provider::SendRequestFilter,
        meta: LogMeta,
    ) -> Result<OdfRequest, InternalError> {
        let id = event.request_id;
        let data: OdfRequestData = ciborium::from_reader(event.request.as_ref()).int_err()?;
        Ok(OdfRequest { id, data, meta })
    }

    #[tracing::instrument(level = "info", skip(self))]
    async fn execute_query(&self, request: &OdfRequest) -> Result<OdfResult, InternalError> {
        // TODO: Handle invalid requests
        let http_response = self
            .api_client
            .get(self.api_query_url.clone())
            .query(&[("query", request.data.sql.as_str())])
            .send()
            .await
            .int_err()?
            .error_for_status()
            .int_err()?;

        let api_response: ApiQueryResponse = http_response.json().await.int_err()?;
        tracing::trace!(?api_response, "Received API response");

        let data = Self::to_cbor_data(api_response.data);

        Ok(OdfResult { data })
    }

    fn to_cbor_data(json_records: Vec<serde_json::Value>) -> Vec<Vec<ciborium::Value>> {
        let mut res = Vec::new();

        for record in json_records {
            res.push(vec![
                Self::to_cbor(&record["province"]),
                Self::to_cbor(&record["count"]),
            ]);
        }

        res
    }

    fn to_cbor(value: &serde_json::Value) -> ciborium::Value {
        match value {
            serde_json::Value::Null => ciborium::Value::Null,
            serde_json::Value::Bool(v) => ciborium::Value::Bool(*v),
            serde_json::Value::Number(v) if v.is_u64() => {
                ciborium::Value::Integer(v.as_u64().unwrap().into())
            }
            serde_json::Value::Number(v) if v.is_i64() => {
                ciborium::Value::Integer(v.as_i64().unwrap().into())
            }
            serde_json::Value::Number(v) if v.is_f64() => {
                ciborium::Value::Float(v.as_f64().unwrap())
            }
            serde_json::Value::Number(_) => unreachable!(),
            serde_json::Value::String(v) => ciborium::Value::Text(v.clone()),
            serde_json::Value::Array(_) => unimplemented!("Nested arrays are not yet supported"),
            serde_json::Value::Object(_) => unimplemented!("Nested structs are not yet supported"),
        }
    }

    #[tracing::instrument(level = "info", skip(self))]
    async fn send_result(&self, request_id: u64, result: OdfResult) -> Result<(), InternalError> {
        let mut result_encoded = Vec::new();
        ciborium::into_writer(&result, &mut result_encoded).int_err()?;

        let transaction: ethers::contract::ContractCall<_, _> = self
            .oracle_contract
            .provide_result(request_id, result_encoded.into())
            .from(self.config.executor_address);

        let pending_tx = match transaction.send().await {
            Ok(tr) => Ok(tr),
            Err(err @ ContractError::Revert(_)) => {
                match err.decode_contract_revert::<IOdfProviderErrors>() {
                    Some(IOdfProviderErrors::RequestNotFound(_)) => {
                        tracing::warn!(
                            "Tried to fulfill already processed request. Transaction reverted."
                        );
                        return Ok(());
                    }
                    // TODO: Handle executor concurrency
                    _ => Err(err.int_err()),
                }
            }
            Err(err) => Err(err.int_err()),
        }?;

        tracing::info!(
            confirmations = self.config.transaction_confirmations,
            retries = self.config.transaction_retries,
            "Waiting transaction to be accepted"
        );

        let mined_tx = pending_tx
            .confirmations(self.config.transaction_confirmations)
            .retries(self.config.transaction_retries)
            .await
            .int_err()?;

        let receipt = mined_tx.unwrap();
        tracing::info!(receipt = ?receipt, "Transaction successful");

        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn run(args: Cli, config: Config) -> Result<(), InternalError> {
    tracing::info!(?args, ?config, "Starting ODF Oracle Executor");

    // Init RPC client
    let rpc_client = Arc::new(Provider::<Http>::connect(config.rpc_url.as_str()).await);
    let chain_id = rpc_client.get_chainid().await.int_err()?;
    let last_block = rpc_client.get_block_number().await.int_err()?;
    tracing::info!(chain_id = %chain_id, last_block = %last_block, "Chain info");

    // Init API client
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = &config.api_access_token {
        let mut auth =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).int_err()?;
        auth.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth);
    }
    let api_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .int_err()?;

    let executor = OdfOracleExecutor::new(config, rpc_client, api_client);
    executor.check_authorized().await?;

    tracing::info!("Entering executor loop");
    executor.run().await?;

    Ok(())
}
