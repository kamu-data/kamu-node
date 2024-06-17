// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;
use opendatafabric::{DatasetID, Multihash};

/////////////////////////////////////////////////////////////////////////////////////////

/// Client interface for making ODF data queries
#[async_trait::async_trait]
pub trait OdfApiClient {
    async fn query(&self, request: QueryRequest) -> Result<QueryResponse, QueryError>;
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    /// SQL query
    pub query: String,

    /// How data should be layed out in the response
    pub data_format: String,

    /// What representation to use for the schema
    pub schema_format: Option<String>,

    /// Mapping between dataset names used in the query and their stable IDs, to
    /// make query resistant to datasets being renamed
    pub aliases: Option<Vec<QueryDatasetAlias>>,

    /// State information used to reproduce query at a specific point in time
    pub as_of_state: Option<QueryState>,

    /// Whether to include schema info about the response
    pub include_schema: bool,

    /// Whether to include dataset state info for query reproducibility
    pub include_state: bool,

    /// Whether to include a logical hash of the resulting data batch.
    /// See: https://docs.kamu.dev/odf/spec/#physical-and-logical-hashes
    pub include_data_hash: bool,

    /// Pagination: skips first N records
    pub skip: Option<u64>,

    /// Pagination: limits number of records in response to N
    pub limit: Option<u64>,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Deserialize)]
pub struct QueryResponse {
    pub data: serde_json::Value,

    #[serde(default)]
    pub schema: Option<String>,

    #[serde(default)]
    pub state: Option<QueryState>,

    #[serde(default)]
    pub data_hash: Option<Multihash>,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDatasetAlias {
    pub alias: String,
    pub id: DatasetID,
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryState {
    pub inputs: Vec<QueryDatasetState>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDatasetState {
    pub id: DatasetID,
    pub block_hash: Multihash,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] InternalError),
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
            _ => Err(QueryError::Internal(
                http_resp.error_for_status().err().unwrap().int_err(),
            )),
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
