// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use internal_error::*;

/////////////////////////////////////////////////////////////////////////////////////////

/// Client interface for making ODF data queries
#[async_trait::async_trait]
pub trait OdfApiClient {
    async fn query(&self, sql: &str) -> Result<QueryResponse, InternalError>;
}

#[derive(Debug, serde::Deserialize)]
pub struct QueryResponse {
    pub data: Vec<Vec<serde_json::Value>>,
}

/////////////////////////////////////////////////////////////////////////////////////////

pub struct OdfApiClientRest {
    client: reqwest::Client,
    query_url: url::Url,
}

#[async_trait::async_trait]
impl OdfApiClient for OdfApiClientRest {
    async fn query(&self, sql: &str) -> Result<QueryResponse, InternalError> {
        let http_response = self
            .client
            .get(self.query_url.clone())
            .query(&[("query", sql), ("format", "json-aoa"), ("schema", "false")])
            .send()
            .await
            .int_err()?
            .error_for_status()
            .int_err()?;

        http_response.json().await.int_err()
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
