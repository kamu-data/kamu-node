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

use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::eth::{Filter, Log};
use alloy::sol_types::{SolEvent, SolEventInterface};
use alloy::transports::BoxTransport;
use internal_error::*;

use crate::api_client::OdfApiClient;
use crate::Config;

/////////////////////////////////////////////////////////////////////////////////////////

// Must be in sync with https://github.com/kamu-data/kamu-contracts/blob/master/src/Odf.sol
alloy::sol! {
    #[sol(rpc)]
    interface IOdfProvider {
        // Emitted when client request was made and awaits a response
        #[derive(Debug)]
        event SendRequest(uint64 indexed requestId, address indexed consumerAddr, bytes request);

        // Emitted when a provider fulfills a pending request
        #[derive(Debug)]
        event ProvideResult(
            uint64 indexed requestId,
            address indexed consumerAddr,
            address indexed providerAddr,
            bytes result,
            bool consumerError,
            bytes consumerErrorData
        );

        // Returned when provider was not registered to provide results to the oracle
        #[derive(Debug)]
        error UnauthorizedProvider(address providerAddr);

        // Returned when pending request by this ID is not found
        #[derive(Debug)]
        error RequestNotFound(uint64 requestId);

        // Returns true/false whether `addr` is authorized to provide results to this oracle
        function canProvideResults(address addr) external view returns (bool);

        // Called to fulfill a pending request
        // See `OdfResponse` for explanation of the `result`
        function provideResult(uint64 requestId, bytes memory result) external;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Debug)]
struct OdfRequest {
    pub id: u64,
    pub data: OdfRequestData,
    pub log: Log<IOdfProvider::SendRequest>,
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

pub struct OdfOracleProvider<P: Provider> {
    config: Config,
    rpc_client: P,
    oracle_contract: IOdfProvider::IOdfProviderInstance<BoxTransport, P>,
    api_client: Arc<dyn OdfApiClient>,
}

impl<P: Provider + Clone> OdfOracleProvider<P> {
    pub fn new(config: Config, rpc_client: P, api_client: Arc<dyn OdfApiClient>) -> Self {
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
            .canProvideResults(self.config.provider_address)
            .from(self.config.provider_address)
            .call()
            .await
        {
            Ok(v) => Ok(v._0),
            Err(err) => Err(err.int_err()),
        }
    }

    /// Check balance of the provider to be able to pay for transactions
    pub async fn get_balance(&self) -> Result<U256, InternalError> {
        self.rpc_client
            .get_balance(self.config.provider_address)
            .await
            .int_err()
    }

    pub async fn run(self) -> Result<(), InternalError> {
        let mut from_block = self.config.oracle_contract_first_block;
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
        let from_block = from_block.unwrap_or(self.config.oracle_contract_first_block);

        let to_block = if let Some(to_block) = to_block {
            to_block
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
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Log<IOdfProvider::SendRequest>>, InternalError> {
        let mut pending_requests = std::collections::HashMap::new();

        let filter = Filter::new()
            .address(self.config.oracle_contract_address)
            .event_signature(vec![
                IOdfProvider::SendRequest::SIGNATURE_HASH,
                IOdfProvider::ProvideResult::SIGNATURE_HASH,
            ])
            .from_block(from_block)
            .to_block(to_block);

        for log in self.rpc_client.get_logs(&filter).await.int_err()? {
            assert!(!log.removed, "Encountered removed log: {log:#?}");

            let log_decoded =
                IOdfProvider::IOdfProviderEvents::decode_log(&log.inner, true).int_err()?;

            tracing::trace!(?log, "Observed log");

            match log_decoded.data {
                IOdfProvider::IOdfProviderEvents::SendRequest(event) => {
                    tracing::debug!(request_id = ?event.requestId, "Adding pending request");
                    pending_requests.insert(
                        event.requestId,
                        Log {
                            inner: alloy::primitives::Log {
                                address: log_decoded.address,
                                data: event,
                            },
                            block_hash: log.block_hash,
                            block_number: log.block_number,
                            block_timestamp: log.block_timestamp,
                            transaction_hash: log.transaction_hash,
                            transaction_index: log.transaction_index,
                            log_index: log.log_index,
                            removed: log.removed,
                        },
                    );
                }
                IOdfProvider::IOdfProviderEvents::ProvideResult(event) => {
                    tracing::debug!(request_id = ?event.requestId, "Removing request as fulfilled");
                    pending_requests.remove(&event.requestId);
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
        requests_batch: Vec<Log<IOdfProvider::SendRequest>>,
    ) -> Result<Vec<OdfResult>, InternalError> {
        let mut results = Vec::new();

        for log in requests_batch {
            // TODO: Handle malformed requests
            let request = Self::decode_request(log)?;
            // TODO: Handle invalid requests
            // TODO: Concurrency
            let result = self.execute_query(request).await?;
            results.push(result);
        }

        Ok(results)
    }

    fn decode_request(log: Log<IOdfProvider::SendRequest>) -> Result<OdfRequest, InternalError> {
        let id = log.inner.requestId;
        let data: OdfRequestData = ciborium::from_reader(log.inner.request.as_ref()).int_err()?;
        Ok(OdfRequest { id, data, log })
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

        let transaction = self
            .oracle_contract
            .provideResult(result.request_id, result_encoded.into())
            .from(self.config.provider_address);

        // TODO: We should ingore RequestNotFound errors as indicating that request was
        // already satisfied by another provider. But getting error data is currently
        // hard with alloy
        // See: https://github.com/alloy-rs/alloy/issues/787
        let pending_tx = match transaction.send().await {
            Ok(tr) => Ok(tr),
            Err(err) => Err(err.int_err()),
        }?;

        tracing::debug!(
            transaction_confirmations = self.config.transaction_confirmations,
            transaction_timeout_s = self.config.transaction_timeout_s,
            "Waiting transaction to be accepted"
        );

        let receipt = pending_tx
            .with_required_confirmations(self.config.transaction_confirmations)
            .with_timeout(Some(Duration::from_secs(self.config.transaction_timeout_s)))
            .get_receipt()
            .await
            .int_err()?;

        tracing::info!(receipt = ?receipt, "Transaction confirmed");

        Ok(())
    }
}
