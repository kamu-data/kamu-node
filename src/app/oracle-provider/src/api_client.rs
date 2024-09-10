// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;
use opendatafabric as odf;

/////////////////////////////////////////////////////////////////////////////////////////

/// Client interface for making ODF data queries
#[async_trait::async_trait]
pub trait OdfApiClient {
    async fn query(&self, request: QueryRequest) -> Result<QueryResponse, QueryError>;
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Separate HTTP API request/response types into a crate to make writing
// clients easier
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    /// Query string
    pub query: String,

    /// Dialect of the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_dialect: Option<QueryDialect>,

    /// How data should be layed out in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_format: Option<DataFormat>,

    /// What information to include
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<Include>,

    /// Optional information used to affix an alias to the specific
    /// [`odf::DatasetID`] and reproduce the query at a specific state in time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datasets: Option<Vec<DatasetState>>,

    /// Pagination: skips first N records
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,

    /// Pagination: limits number of records in response to N
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    /// Inputs that can be used to fully reproduce the query
    #[serde(default)]
    pub input: Option<QueryRequest>,

    /// Query results
    pub output: Outputs,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outputs {
    /// Resulting data
    pub data: serde_json::Value,

    /// How data is layed out in the response
    pub data_format: DataFormat,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum QueryDialect {
    SqlDataFusion,
    SqlFlink,
    SqlRisingWave,
    SqlSpark,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum::Display,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum(serialize_all = "PascalCase")]
#[strum(ascii_case_insensitive)]
pub enum Include {
    /// Include input block that can be used to fully reproduce the query
    #[serde(alias = "input")]
    Input,

    /// Include cryptographic proof that lets you hold the node accountable for
    /// the response
    #[serde(alias = "proof")]
    Proof,

    /// Include schema of the data query resulted in
    #[serde(alias = "schema")]
    Schema,
}

/////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DataFormat {
    #[default]
    #[serde(alias = "jsonaos")]
    #[serde(alias = "json-aos")]
    JsonAos,
    #[serde(alias = "jsonsoa")]
    #[serde(alias = "json-soa")]
    JsonSoa,
    #[serde(alias = "jsonaoa")]
    #[serde(alias = "json-aoa")]
    JsonAoa,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetState {
    /// Globally unique identity of the dataset
    pub id: odf::DatasetID,

    /// Alias to be used in the query
    pub alias: String,

    /// Last block hash of the input datasets that was or should be considered
    /// during the query planning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<odf::Multihash>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    ApiRequestError(ApiRequestError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[derive(Debug, thiserror::Error)]
#[error("Api request error status code {status} body: {body:?}")]
pub struct ApiRequestError {
    pub status: reqwest::StatusCode,
    pub body: Option<String>,
}

/////////////////////////////////////////////////////////////////////////////////////////

pub struct OdfApiClientRest {
    client: reqwest::Client,
    query_url: url::Url,
}

#[async_trait::async_trait]
impl OdfApiClient for OdfApiClientRest {
    async fn query(&self, request: QueryRequest) -> Result<QueryResponse, QueryError> {
        let http_resp = self
            .client
            .post(self.query_url.clone())
            .json(&request)
            .send()
            .await
            .int_err()?;

        match http_resp.status() {
            reqwest::StatusCode::OK => Ok(http_resp.json().await.int_err()?),
            reqwest::StatusCode::BAD_REQUEST => {
                let body = http_resp.text().await.int_err()?;
                Err(QueryError::BadRequest(body))
            }
            reqwest::StatusCode::NOT_FOUND => {
                let body = http_resp.text().await.int_err()?;
                Err(QueryError::DatasetNotFound(body))
            }
            _ => {
                let status = http_resp.status();
                let body = http_resp.text().await.ok();
                Err(QueryError::ApiRequestError(ApiRequestError {
                    status,
                    body,
                }))
            }
        }
    }
}

impl OdfApiClientRest {
    pub fn new(url: url::Url, access_token: Option<String>) -> Result<Self, InternalError> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(access_token) = access_token {
            let mut auth =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {access_token}"))
                    .int_err()?;
            auth.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .int_err()?;

        let query_url =
            url::Url::parse(&format!("{}/query", url.as_str().trim_end_matches('/'))).unwrap();

        Ok(Self { client, query_url })
    }
}
