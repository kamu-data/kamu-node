// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::Future;
use std::path::PathBuf;

use internal_error::{InternalError, ResultIntoInternal};
use kamu_cli_e2e_common::KamuApiServerClient;
use kamu_node_puppet::extensions::ServerOutput;
use reqwest::Url;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

use crate::{KamuFlightSQLClient, KamuFlightSQLClientExt};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn api_server_e2e_test<ServerRunFut, Fixture, FixtureFut>(
    e2e_data_file_path: PathBuf,
    workspace_path: PathBuf,
    server_run_fut: ServerRunFut,
    fixture: Fixture,
) where
    ServerRunFut: Future<Output = ServerOutput>,
    Fixture: FnOnce(KamuApiServerClient) -> FixtureFut,
    FixtureFut: Future<Output = ()> + Send + 'static,
{
    let test_fut = async move {
        let (base_url, _) = get_server_base_urls(e2e_data_file_path).await?;
        let kamu_api_server_client = KamuApiServerClient::new(base_url, workspace_path);

        kamu_api_server_client.e2e().ready().await?;

        let fixture_res = {
            // tokio::spawn() is used to catch panic, otherwise the test will hang
            tokio::spawn(fixture(kamu_api_server_client.clone())).await
        };

        kamu_api_server_client.e2e().shutdown().await?;

        fixture_res.int_err()
    };

    let (server_output, test_res) = tokio::join!(server_run_fut, test_fut);

    if let Err(e) = test_res {
        let mut panic_message = format!("Fixture execution error:\n{e:?}\n");

        panic_message += "Server output:\n";
        panic_message += "stdout:\n";
        panic_message += server_output.stdout.as_str();
        panic_message += "\n";
        panic_message += "stderr:\n";
        panic_message += server_output.stderr.as_str();
        panic_message += "\n";

        panic!("{panic_message}");
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn api_flight_sql_e2e_test<ServerRunFut, Fixture, FixtureFut>(
    e2e_data_file_path: PathBuf,
    workspace_path: PathBuf,
    server_run_fut: ServerRunFut,
    fixture: Fixture,
) where
    ServerRunFut: Future<Output = ServerOutput>,
    Fixture: FnOnce(KamuFlightSQLClient) -> FixtureFut,
    FixtureFut: Future<Output = ()> + Send + 'static,
{
    let test_fut = async move {
        let (api_server_base_url, flight_sql_base_url) =
            get_server_base_urls(e2e_data_file_path).await?;
        let kamu_api_server_client = KamuApiServerClient::new(api_server_base_url, workspace_path);

        let kamu_flight_sql_server_client = kamu_api_server_client
            .flight_sql_client(flight_sql_base_url)
            .await;

        kamu_api_server_client.e2e().ready().await?;

        let fixture_res = {
            // tokio::spawn() is used to catch panic, otherwise the test will hang
            tokio::spawn(fixture(kamu_flight_sql_server_client.clone())).await
        };

        kamu_api_server_client.e2e().shutdown().await?;

        fixture_res.int_err()
    };

    let (server_output, test_res) = tokio::join!(server_run_fut, test_fut);

    if let Err(e) = test_res {
        let mut panic_message = format!("Fixture execution error:\n{e:?}\n");

        panic_message += "Server output:\n";
        panic_message += "stdout:\n";
        panic_message += server_output.stdout.as_str();
        panic_message += "\n";
        panic_message += "stderr:\n";
        panic_message += server_output.stderr.as_str();
        panic_message += "\n";

        panic!("{panic_message}");
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Function returns a tuple of API server base URL and Flight SQL server base
/// URL which are stored in a file at `e2e-output-data-path`.
/// First line is API server base URL, second line is Flight SQL server base
/// URL.
async fn get_server_base_urls(e2e_data_file_path: PathBuf) -> Result<(Url, Url), InternalError> {
    let retry_strategy = FixedInterval::from_millis(500).take(10);
    let base_urls = Retry::spawn(retry_strategy, || async {
        let data = tokio::fs::read_to_string(e2e_data_file_path.clone())
            .await
            .int_err()?;
        let (api_server_base_url, flight_sql_base_url) = data.as_str().split_once('\n').unwrap();

        Ok((
            Url::parse(api_server_base_url).unwrap(),
            Url::parse(flight_sql_base_url).unwrap(),
        ))
    })
    .await?;

    Ok(base_urls)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
