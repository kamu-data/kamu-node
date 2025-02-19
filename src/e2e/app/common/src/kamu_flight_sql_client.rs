// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use arrow_flight::sql::client::FlightSqlServiceClient;
use async_trait::async_trait;
use datafusion::prelude::SessionContext;
use futures::TryStreamExt;
use kamu_cli_e2e_common::KamuApiServerClient;
use reqwest::Url;
use tonic::transport::Channel;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type AccessToken = String;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum RequestBody {
    Json(serde_json::Value),
    NdJson(String),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ExpectedResponseBody {
    Json(serde_json::Value),
    Plain(String),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait KamuFlightSQLClientExt {
    async fn flight_sql_client(&self, base_url: Url) -> KamuFlightSQLClient;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl KamuFlightSQLClientExt for KamuApiServerClient {
    async fn flight_sql_client(&self, base_url: Url) -> KamuFlightSQLClient {
        KamuFlightSQLClient::new(self.clone(), base_url).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct KamuFlightSQLClient {
    flight_sql_client: FlightSqlServiceClient<Channel>,
    server_base_url: Url,
    kamu_api_client: KamuApiServerClient,
}

impl KamuFlightSQLClient {
    pub async fn new(kamu_api_client: KamuApiServerClient, server_base_url: Url) -> Self {
        let channel_url = server_base_url.to_string();
        let channel = Channel::from_shared(channel_url)
            .unwrap()
            .connect()
            .await
            .expect("error connecting");
        let flight_sql_client = FlightSqlServiceClient::new(channel);

        Self {
            flight_sql_client,
            server_base_url,
            kamu_api_client,
        }
    }

    pub fn set_token(&mut self, token: AccessToken) {
        self.flight_sql_client.set_token(token);
    }

    pub async fn set_anonymous(&mut self) {
        self.flight_sql_client
            .handshake("anonymous", "")
            .await
            .unwrap();
    }

    pub fn kamu_api_client(&mut self) -> &mut KamuApiServerClient {
        &mut self.kamu_api_client
    }

    pub fn get_base_url(&self) -> &Url {
        &self.server_base_url
    }

    pub async fn flight_sql_assert_call(
        &mut self,
        query: &str,
        expected_schema_maybe: Option<&str>,
        expected_data_maybe: Option<&str>,
    ) {
        let fi = self
            .flight_sql_client
            .execute(query.to_owned(), None)
            .await
            .unwrap();

        let mut record_batches: Vec<_> = self
            .flight_sql_client
            .do_get(fi.endpoint[0].ticket.clone().unwrap())
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap();

        let ctx = SessionContext::new();
        let df = ctx.read_batch(record_batches.pop().unwrap()).unwrap();

        if let Some(expected_schema) = expected_schema_maybe {
            odf::utils::testing::assert_schema_eq(df.schema(), expected_schema);
        }
        if let Some(expected_data) = expected_data_maybe {
            odf::utils::testing::assert_data_eq(df, expected_data).await;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
