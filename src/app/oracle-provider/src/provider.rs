// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;
use std::time::Duration;

use ethers::prelude::*;
use internal_error::*;

use crate::api_client::OdfApiClient;
use crate::Config;

/////////////////////////////////////////////////////////////////////////////////////////

abigen!(
    IOdfProvider,
    "./abi/IOdfProvider.json",
    event_derives(serde::Deserialize, serde::Serialize),
);

/////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
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

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct OdfResult {
    pub request_id: u64,
    pub data: OdfResultData,
}

#[derive(Debug, serde::Serialize)]
struct OdfResultData {
    pub data: Vec<Vec<ciborium::Value>>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error("Provider is not authorized to provide results to the oracle contract")]
pub struct ProviderUnauthorized;

/////////////////////////////////////////////////////////////////////////////////////////

pub struct OdfOracleProvider<M: Middleware> {
    config: Config,
    rpc_client: Arc<M>,
    oracle_contract: IOdfProvider<M>,
    api_client: Arc<dyn OdfApiClient>,
}

impl<M: Middleware + 'static> OdfOracleProvider<M> {
    pub fn new(config: Config, rpc_client: Arc<M>, api_client: Arc<dyn OdfApiClient>) -> Self {
        let oracle_contract = IOdfProvider::new(config.oracle_contract_address, rpc_client.clone());

        Self {
            config,
            rpc_client,
            api_client,
            oracle_contract,
        }
    }

    /// Check whether the provider is authorized to submit results
    pub async fn is_authorized(&self) -> Result<bool, InternalError> {
        match self
            .oracle_contract
            .can_provide_results(self.config.provider_address)
            .from(self.config.provider_address)
            .call()
            .await
        {
            Ok(v) => Ok(v),
            Err(err) => Err(err.int_err()),
        }
    }

    /// Check balance of the provider to be able to pay for transactions
    pub async fn get_balance(&self) -> Result<U256, InternalError> {
        self.rpc_client
            .get_balance(self.config.provider_address, None)
            .await
            .int_err()
    }

    pub async fn run(self) -> Result<(), InternalError> {
        let mut from_block = ethers::types::U64::from(self.config.oracle_contract_first_block);
        let mut idle_start = None;

        // Pre-flight loop: Wait until we have basic pre-requisites to function
        self.wait_for_auth_and_balance().await?;

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

            // TODO: Refactor towards concurrent streams model where blockchain scanning
            // continues independently from execution and submitting
            // transactions
            let pending_requests = self.scan_block_range(from_block, last_block).await?;
            let results = self.process_request_batch(pending_requests).await?;
            self.send_results(results).await?;

            // TODO: Chain reorg resistance
            from_block = last_block + 1;
        }
    }

    pub async fn run_once(
        self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<(), InternalError> {
        let from_block =
            ethers::types::U64::from(from_block.unwrap_or(self.config.oracle_contract_first_block));

        let to_block = if let Some(to_block) = to_block {
            ethers::types::U64::from(to_block)
        } else {
            self.rpc_client.get_block_number().await.int_err()?
        };

        // TODO: Reorg resistance
        if from_block > to_block {
            return Ok(());
        }

        let pending_requests = self.scan_block_range(from_block, to_block).await?;
        let results = self.process_request_batch(pending_requests).await?;
        self.send_results(results).await?;

        Ok(())
    }

    async fn wait_for_auth_and_balance(&self) -> Result<(), InternalError> {
        let mut first = true;
        loop {
            if self.is_authorized().await? {
                break;
            }
            if first {
                tracing::warn!(
                    "Provider is unauthorized to provide results - waiting for permissions"
                );
                first = false;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        let mut first = true;
        loop {
            let balance = self.get_balance().await?;
            if !balance.is_zero() {
                tracing::info!(balance = %balance, "Provider balance on start");
                break;
            }
            if first {
                tracing::warn!(
                    "Provider has zero balance - waiting for some tokens to be able to submit \
                     transactions"
                );
                first = false;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok(())
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

            tracing::trace!(?event, "Observed event");

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

    #[tracing::instrument(level = "info", skip_all)]
    async fn process_request_batch(
        &self,
        requests_batch: Vec<(i_odf_provider::SendRequestFilter, ethers::contract::LogMeta)>,
    ) -> Result<Vec<OdfResult>, InternalError> {
        let mut results = Vec::new();

        for (event, meta) in requests_batch {
            // TODO: Handle malformed requests
            let request = Self::decode_request(event, meta)?;
            // TODO: Handle invalid requests
            // TODO: Concurrency
            let result = self.execute_query(request).await?;
            results.push(result);
        }

        Ok(results)
    }

    fn decode_request(
        event: i_odf_provider::SendRequestFilter,
        meta: LogMeta,
    ) -> Result<OdfRequest, InternalError> {
        let id = event.request_id;
        let data: OdfRequestData = ciborium::from_reader(event.request.as_ref()).int_err()?;
        Ok(OdfRequest { id, data, meta })
    }

    #[tracing::instrument(level = "debug", skip_all, fields(request_id = request.id))]
    async fn execute_query(&self, request: OdfRequest) -> Result<OdfResult, InternalError> {
        tracing::debug!(?request, "Executing API query");

        let api_response = self.api_client.query(request.data.sql.as_str()).await?;
        tracing::debug!(?api_response, "API response");

        let data = super::cbor::records_json_to_cbor(api_response.data);

        Ok(OdfResult {
            request_id: request.id,
            data: OdfResultData { data },
        })
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn send_results(&self, results: Vec<OdfResult>) -> Result<(), InternalError> {
        for result in results {
            // TODO: Concurrency
            // TODO: Handle failed transactions
            self.send_result(result).await?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn send_result(&self, result: OdfResult) -> Result<(), InternalError> {
        let mut result_encoded = Vec::new();
        ciborium::into_writer(&result.data, &mut result_encoded).int_err()?;

        let transaction: ethers::contract::ContractCall<_, _> = self
            .oracle_contract
            .provide_result(result.request_id, result_encoded.into())
            .from(self.config.provider_address);

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
                    Some(IOdfProviderErrors::UnauthorizedProvider(_)) => {
                        Err(ProviderUnauthorized.int_err())
                    }
                    _ => Err(err.int_err()),
                }
            }
            Err(err) => Err(err.int_err()),
        }?;

        tracing::debug!(
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
        tracing::info!(receipt = ?receipt, "Transaction confirmed");

        Ok(())
    }
}
