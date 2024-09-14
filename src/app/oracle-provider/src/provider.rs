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

use alloy::eips::BlockNumberOrTag;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::eth::{Filter, Log};
use alloy::sol_types::{SolEvent, SolEventInterface};
use alloy::transports::BoxTransport;
use chrono::{DateTime, Utc};
use internal_error::*;
use opendatafabric::{DatasetID, Multihash};
use tracing::Instrument;

use crate::api_client::*;
use crate::Config;

/////////////////////////////////////////////////////////////////////////////////////////

// Must be in sync with https://github.com/kamu-data/kamu-contracts/blob/master/src/Odf.sol
alloy::sol! {
    #[sol(rpc)]
    interface IOdfProvider {
        // Emitted when client request was made and awaits a response
        #[derive(Debug)]
        event SendRequest(uint64 indexed requestId, address indexed consumerAddr, bytes request);

        // Emitted when a provider fulfills a pending request.
        //
        // Fields:
        // - requestId - unique identifier of the request
        // - consumerAddr - address of the contract that sent request and awaits the response
        // - providerAddr - address of the provider that fulfilled the request
        // - response - response data, see `OdfResponse`
        // - requestError - indicates that response contained an unrecoverable error instead of data
        // - consumerError - indicates that consumer callback has failed when processing the result
        // - consumerErrorData - will contain the details of consumer-side error
        #[derive(Debug)]
        event ProvideResult(
            uint64 indexed requestId,
            address indexed consumerAddr,
            address indexed providerAddr,
            bytes response,
            bool requestError,
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
        function provideResult(uint64 requestId, bytes calldata result) external;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Debug)]
struct OdfRequest {
    pub id: u64,
    pub sql: String,
    pub aliases: Vec<(String, DatasetID)>,
    pub log: Log<IOdfProvider::SendRequest>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct OdfResult {
    pub request_id: u64,
    pub inner: Result<OdfResultOk, OdfResultErr>,
}

#[derive(Debug)]
struct OdfResultOk {
    pub data: serde_json::Value,
    pub state: Vec<(DatasetID, Multihash)>,
}

#[derive(Debug)]
struct OdfResultErr {
    pub error_message: String,
}

impl OdfResult {
    const VERSION: u16 = 1;

    pub fn into_cbor(self) -> ciborium::Value {
        use ciborium::Value as V;

        match self.inner {
            Ok(v) => {
                // Flatten the state
                let mut state = Vec::new();
                for (id, block_hash) in v.state {
                    state.push(V::Bytes(id.as_bytes().to_vec()));
                    state.push(V::Bytes(block_hash.as_bytes().to_vec()));
                }

                V::Array(vec![
                    V::Integer(Self::VERSION.into()),
                    V::Bool(true),
                    super::cbor::json_to_cbor(v.data),
                    V::Bytes(Vec::new()),
                    V::Array(state),
                ])
            }
            Err(v) => V::Array(vec![
                V::Integer(Self::VERSION.into()),
                V::Bool(false),
                V::Text(v.error_message),
            ]),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error("Provider is not authorized to provide results to the oracle contract")]
pub struct ProviderUnauthorized;

/////////////////////////////////////////////////////////////////////////////////////////

pub struct OdfOracleProviderMetrics {
    pub wallet_balance: prometheus::Gauge,
    pub api_queries_num: prometheus::IntCounter,
    pub transactions_num: prometheus::IntCounter,
}

impl OdfOracleProviderMetrics {
    pub fn new(chain_id: u64, node_host: &str) -> Self {
        use prometheus::*;

        Self {
            wallet_balance: Gauge::with_opts(
                Opts::new("wallet_balance_wei", "Balance of the provider's wallet")
                    .const_label("chain_id", chain_id.to_string()),
            )
            .unwrap(),
            api_queries_num: IntCounter::with_opts(
                Opts::new("api_queries_total", "ODF API queries executed")
                    .const_label("chain_id", chain_id.to_string())
                    .const_label("node_host", node_host),
            )
            .unwrap(),
            transactions_num: IntCounter::with_opts(
                Opts::new(
                    "transactions_submitted_total",
                    "Chain transactions submitted",
                )
                .const_label("chain_id", chain_id.to_string()),
            )
            .unwrap(),
        }
    }

    pub fn register(&self, reg: &prometheus::Registry) -> Result<(), prometheus::Error> {
        reg.register(Box::new(self.wallet_balance.clone()))?;
        reg.register(Box::new(self.api_queries_num.clone()))?;
        reg.register(Box::new(self.transactions_num.clone()))?;
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub struct OdfOracleProvider<P: Provider> {
    config: Config,
    rpc_client: P,
    oracle_contract: IOdfProvider::IOdfProviderInstance<BoxTransport, P>,
    api_client: Arc<dyn OdfApiClient>,
    metrics: OdfOracleProviderMetrics,
}

impl<P: Provider + Clone> OdfOracleProvider<P> {
    pub fn new(
        config: Config,
        rpc_client: P,
        api_client: Arc<dyn OdfApiClient>,
        metrics: OdfOracleProviderMetrics,
    ) -> Self {
        let oracle_contract = IOdfProvider::new(config.oracle_contract_address, rpc_client.clone());

        Self {
            config,
            rpc_client,
            api_client,
            oracle_contract,
            metrics,
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
        let balance = self
            .rpc_client
            .get_balance(self.config.provider_address)
            .await
            .int_err()?;

        let balance_fp: f64 = balance.to_string().parse().unwrap();
        self.metrics.wallet_balance.set(balance_fp);

        Ok(balance)
    }

    /// Determines the block to start scanning from based on multiple config
    /// options
    pub async fn get_starting_block(&self) -> Result<u64, InternalError> {
        if let Some(scan_from_block) = self.config.scan_from_block {
            Ok(scan_from_block)
        } else if let Some(scan_last_blocks) = self.config.scan_last_blocks {
            let latest_block = self.rpc_client.get_block_number().await.int_err()?;
            Ok(latest_block.saturating_sub(scan_last_blocks))
        } else if let Some(scan_last_blocks_period) = self.config.scan_last_blocks_period {
            let lookback: Duration = scan_last_blocks_period.into();
            self.get_approx_block_number_by_time(Utc::now() - lookback)
                .await
        } else {
            Err("Config does not specify the scanning interval".int_err())
        }
    }

    pub async fn get_approx_block_number_by_time(
        &self,
        time: DateTime<Utc>,
    ) -> Result<u64, InternalError> {
        let latest_block = self
            .rpc_client
            .get_block_by_number(BlockNumberOrTag::Latest, false)
            .await
            .int_err()?
            .ok_or("Could not read latest block".int_err())?;

        let latest_block_number = latest_block.header.number.unwrap();

        // Jump back N blocks to calculate average hash rate
        let jump_back = 1_000;
        if latest_block_number < jump_back {
            Err(format!(
                "With {latest_block_number} blocks there is not enough history to estimate the \
                 hash rate"
            )
            .int_err())?;
        }
        let jump_block = self
            .rpc_client
            .get_block_by_number(
                BlockNumberOrTag::Number(latest_block_number - jump_back),
                false,
            )
            .await
            .int_err()?
            .ok_or("Could not read block".int_err())?;

        let seconds_per_block = ((latest_block.header.timestamp - jump_block.header.timestamp)
            as f64)
            / (jump_back as f64);

        let target_timestamp = u64::try_from(time.timestamp()).unwrap();
        if target_timestamp > latest_block.header.timestamp {
            Err(format!(
                "Target time is in the future: {target_timestamp} > {}",
                latest_block.header.timestamp
            )
            .int_err())?;
        }

        let approx_block_number = latest_block_number
            - (f64::floor(
                ((latest_block.header.timestamp - target_timestamp) as f64) / seconds_per_block,
            ) as u64);

        let target_block = self
            .rpc_client
            .get_block_by_number(BlockNumberOrTag::Number(approx_block_number), false)
            .await
            .int_err()?
            .ok_or("Could not read block".int_err())?;

        tracing::info!(
            target_timestamp,
            latest_block_number,
            latest_block_timestamp = latest_block.header.timestamp,
            jump_block_number = latest_block_number - jump_back,
            jump_block_timestamp = jump_block.header.timestamp,
            seconds_per_block,
            approx_block_number,
            actual_timestamp = target_block.header.timestamp,
            off_by_seconds = u64::abs_diff(target_block.header.timestamp, target_timestamp),
            "Calculated approximate block number by time",
        );

        Ok(approx_block_number)
    }

    pub async fn run(self) -> Result<(), InternalError> {
        let mut from_block = self.get_starting_block().await?;
        let mut idle_start = None;

        // Pre-flight loop: Wait until we have basic pre-requisites to function
        self.wait_for_auth_and_balance().await?;

        loop {
            // TODO: Operate on blocks that have >N confirmations to avoid running into too
            // many reorgs?
            let to_block = self.rpc_client.get_block_number().await.int_err()?;

            // TODO: Reorg resistance
            if from_block > to_block {
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

            let span =
                observability::tracing::root_span!("process_block_range", from_block, to_block);

            self.process_block_range(from_block, to_block)
                .instrument(span)
                .await?;

            // TODO: Chain reorg resistance
            from_block = to_block + 1;
        }
    }

    pub async fn run_once(
        self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<(), InternalError> {
        let from_block = if let Some(from_block) = from_block {
            from_block
        } else {
            self.get_starting_block().await?
        };

        let to_block = if let Some(to_block) = to_block {
            to_block
        } else {
            self.rpc_client.get_block_number().await.int_err()?
        };

        // TODO: Reorg resistance
        if from_block > to_block {
            return Ok(());
        }

        self.process_block_range(from_block, to_block).await?;

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

    async fn process_block_range(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<(), InternalError> {
        // TODO: Refactor towards concurrent streams model where blockchain scanning
        // continues independently from execution and submitting
        // transactions
        let pending_requests = self.scan_block_range(from_block, to_block).await?;
        if !pending_requests.is_empty() {
            let results = self.process_request_batch(pending_requests).await?;
            self.send_results(results).await?;
        }
        Ok(())
    }

    // TODO: This code is much more complex than it should be because of the issue
    // in ethers where calling `contract.events()` sets the address filter but
    // not the topic0 filter, thus when used on an interface rather than final
    // contract - there might be other events present in result and decoding
    // will fail.
    // See: https://github.com/gakonst/ethers-rs/issues/2497
    #[tracing::instrument(level = "info", skip_all)]
    async fn scan_block_range(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Log<IOdfProvider::SendRequest>>, InternalError> {
        assert!(from_block <= to_block);

        let mut pending_requests = std::collections::HashMap::new();

        let mut filter = Filter::new()
            .address(self.config.oracle_contract_address)
            .event_signature(vec![
                IOdfProvider::SendRequest::SIGNATURE_HASH,
                IOdfProvider::ProvideResult::SIGNATURE_HASH,
            ]);

        let mut from_block_page = from_block;

        while from_block_page <= to_block {
            let to_block_page = u64::min(to_block, from_block_page + self.config.blocks_stride);

            tracing::info!(
                from_block = from_block_page,
                to_block = to_block_page,
                "Getting logs page",
            );
            filter = filter.from_block(from_block_page).to_block(to_block_page);
            let logs = self.rpc_client.get_logs(&filter).await.int_err()?;

            for log in logs {
                assert!(!log.removed, "Encountered removed log: {log:#?}");

                let log_decoded =
                    IOdfProvider::IOdfProviderEvents::decode_log(&log.inner, true).int_err()?;

                tracing::trace!(?log, "Observed log");

                match log_decoded.data {
                    IOdfProvider::IOdfProviderEvents::SendRequest(event)
                        if self.config.ignore_requests.contains(&event.requestId)
                            || self.config.ignore_consumers.contains(&event.consumerAddr) =>
                    {
                        tracing::debug!(request_id = ?event.requestId, "Ignoring request as per configuration");
                    }
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

            from_block_page = to_block_page + 1;
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
            if let Some(result) = self.execute_query(request).await? {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Request layout in CBOR is:
    /// [
    ///   version,
    ///   "ds", "alias1", "did:odf:...",
    ///   "ds", "alias2", "did:odf:...",
    ///   "sql", "select ...",
    ///   ...
    /// ]
    fn decode_request(log: Log<IOdfProvider::SendRequest>) -> Result<OdfRequest, InternalError> {
        let id = log.inner.requestId;
        let raw: Vec<ciborium::Value> =
            ciborium::from_reader(log.inner.request.as_ref()).int_err()?;

        tracing::debug!(?raw, "Parsing raw CBOR request");

        let mut raw = raw.into_iter();

        let Some(ciborium::Value::Integer(version)) = raw.next() else {
            Err("Request does not start with version specifier".int_err())?
        };
        if u8::try_from(version) != Ok(1) {
            Err(format!("Unsupported protocol version {version:?}").int_err())?
        }

        let mut sql = None;
        let mut aliases = Vec::new();

        while let Some(key) = raw.next() {
            let ciborium::Value::Text(key) = key else {
                Err("Expected a key".int_err())?
            };
            match key.as_str() {
                "ds" => {
                    let Some(ciborium::Value::Text(alias)) = raw.next() else {
                        Err("Expected an alias".int_err())?
                    };
                    let Some(ciborium::Value::Bytes(did)) = raw.next() else {
                        Err("Expected a dataset ID".int_err())?
                    };
                    let Ok(did) = DatasetID::from_bytes(&did) else {
                        Err("Expected DID bytes".int_err())?
                    };
                    aliases.push((alias, did));
                }
                "sql" => {
                    let Some(ciborium::Value::Text(query)) = raw.next() else {
                        Err("Expected a query".int_err())?
                    };
                    sql = Some(query);
                }
                _ => Err(format!("Unknown key {key}").int_err())?,
            }
        }

        let Some(sql) = sql else {
            Err("Request does not specify a query".int_err())?
        };

        Ok(OdfRequest {
            id,
            sql,
            aliases,
            log,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, fields(request_id = request.id))]
    async fn execute_query(&self, request: OdfRequest) -> Result<Option<OdfResult>, InternalError> {
        tracing::debug!(?request, "Executing API query");

        let rest_request = QueryRequest {
            include: vec![Include::Input],
            query: request.sql,
            query_dialect: Some(QueryDialect::SqlDataFusion),
            data_format: Some(DataFormat::JsonAoa),
            datasets: Some(
                request
                    .aliases
                    .into_iter()
                    .map(|(alias, id)| DatasetState {
                        alias,
                        id,
                        block_hash: None,
                    })
                    .collect(),
            ),
            // TODO: Pagination / detect limits
            skip: None,
            limit: None,
        };

        self.metrics.api_queries_num.inc();

        match self.api_client.query(rest_request).await {
            Ok(rest_response) => {
                tracing::debug!(?rest_response, "Writing successful response");
                Ok(Some(OdfResult {
                    request_id: request.id,
                    inner: Ok(OdfResultOk {
                        data: rest_response.output.data,
                        state: rest_response
                            .input
                            .unwrap()
                            .datasets
                            .unwrap_or_default()
                            .into_iter()
                            .map(|i| (i.id, i.block_hash.unwrap()))
                            .collect(),
                    }),
                }))
            }
            Err(QueryError::BadRequest(msg)) => {
                tracing::warn!("Writing unsuccessful response");
                Ok(Some(OdfResult {
                    request_id: request.id,
                    inner: Err(OdfResultErr { error_message: msg }),
                }))
            }
            Err(QueryError::DatasetNotFound(info)) => {
                tracing::info!(info, "Ignoring request for unknown dataset(s)");
                Ok(None)
            }
            Err(QueryError::ApiRequestError(err)) => {
                tracing::error!(
                    error = ?err,
                    error_msg = %err,
                    "API query failed",
                );
                Err(err.int_err())
            }
            Err(QueryError::Internal(err)) => {
                tracing::error!(
                    error = ?err,
                    error_msg = %err,
                    "API query failed",
                );
                Err(err)
            }
        }
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
        let request_id = result.request_id;

        let mut result_encoded = Vec::new();
        ciborium::into_writer(&result.into_cbor(), &mut result_encoded).int_err()?;

        tracing::debug!(result_hex = hex::encode(&result_encoded), "Encoded result");

        let transaction = self
            .oracle_contract
            .provideResult(request_id, result_encoded.into())
            .from(self.config.provider_address);

        self.metrics.transactions_num.inc();

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

        // Fetch balance to update metric
        self.get_balance().await?;

        Ok(())
    }
}
