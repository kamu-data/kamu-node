// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::Future;

use kamu_cli_e2e_common::KamuApiServerClient;
use kamu_node_puppet::extensions::KamuNodePuppetExt;
use kamu_node_puppet::{KamuNodePuppet, NewWorkspaceOptions};
use regex::Regex;
use sqlx::{PgPool, SqlitePool};

use crate::{api_flight_sql_e2e_test, api_server_e2e_test, KamuFlightSQLClient};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct KamuNodeApiServerHarnessOptions {
    env_vars: Vec<(String, String)>,
    kamu_api_server_config: Option<String>,
}

impl KamuNodeApiServerHarnessOptions {
    pub fn with_kamu_api_config(mut self, content: &str) -> Self {
        self.kamu_api_server_config = Some(content.into());

        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct KamuNodeApiServerHarness {
    options: KamuNodeApiServerHarnessOptions,
}

impl KamuNodeApiServerHarness {
    pub fn postgres(pg_pool: &PgPool, options: KamuNodeApiServerHarnessOptions) -> Self {
        let db = pg_pool.connect_options();
        let kamu_api_config = indoc::formatdoc!(
            r#"
            database:
                provider: postgres
                host: {host}
                port: {port}
                credentialsPolicy:
                    source:
                        kind: rawPassword
                        userName: {user}
                        rawPassword: {password}
                databaseName: {database}
            "#,
            host = db.get_host(),
            port = db.get_port(),
            user = db.get_username(),
            password = db.get_username(), // It's intended: password is same as user for tests
            database = db.get_database().unwrap(),
        );

        Self::new(options, Some(kamu_api_config))
    }

    pub fn sqlite(sqlite_pool: &SqlitePool, options: KamuNodeApiServerHarnessOptions) -> Self {
        // Ugly way to get the path as the settings have a not-so-good signature:
        // SqliteConnectOptions::get_filename(self) -> Cow<'static, Path>
        //                                    ^^^^
        // Arc<T> + consuming = bad combo
        let database_path = {
            let re = Regex::new(r#"filename: "(.*)""#).unwrap();
            let connect_options = format!("{:#?}", sqlite_pool.connect_options());
            let re_groups = re.captures(connect_options.as_str()).unwrap();
            let relative_path = re_groups[1].to_string();

            std::fs::canonicalize(relative_path)
                .unwrap()
                .display()
                .to_string()
        };

        let kamu_api_config = indoc::formatdoc!(
            r#"
            database:
                provider: sqlite
                databasePath: {path}
            "#,
            path = database_path
        );

        Self::new(options, Some(kamu_api_config))
    }

    fn new(
        mut options: KamuNodeApiServerHarnessOptions,
        generated_kamu_api_config: Option<String>,
    ) -> Self {
        let target_config =
            generated_kamu_api_config.map(|target| serde_yaml::from_str(&target).unwrap());
        let source_config = options
            .kamu_api_server_config
            .map(|source| serde_yaml::from_str(&source).unwrap());

        options.kamu_api_server_config = merge_yaml(target_config, source_config)
            .map(|yaml| serde_yaml::to_string(&yaml).unwrap());

        Self { options }
    }

    pub async fn run_api_server<Fixture, FixtureResult>(self, fixture: Fixture)
    where
        Fixture: FnOnce(KamuApiServerClient) -> FixtureResult,
        FixtureResult: Future<Output = ()> + Send + 'static,
    {
        let kamu_api_server = self.into_kamu_api_server();

        let e2e_data_file_path = kamu_api_server.get_e2e_output_data_path();
        let workspace_path = kamu_api_server.workspace_path().to_path_buf();
        let server_run_fut = kamu_api_server.start_api_server(e2e_data_file_path.clone());

        api_server_e2e_test(e2e_data_file_path, workspace_path, server_run_fut, fixture).await;
    }

    pub async fn run_flight_sql_server<Fixture, FixtureResult>(self, fixture: Fixture)
    where
        Fixture: FnOnce(KamuFlightSQLClient) -> FixtureResult,
        FixtureResult: Future<Output = ()> + Send + 'static,
    {
        let kamu_api_server = self.into_kamu_api_server();

        let e2e_data_file_path = kamu_api_server.get_e2e_output_data_path();
        let workspace_path = kamu_api_server.workspace_path().to_path_buf();
        let server_run_fut = kamu_api_server.start_api_server(e2e_data_file_path.clone());

        api_flight_sql_e2e_test(e2e_data_file_path, workspace_path, server_run_fut, fixture).await;
    }

    pub async fn execute_command<Fixture, FixtureResult>(self, fixture: Fixture)
    where
        Fixture: FnOnce(KamuNodePuppet) -> FixtureResult,
        FixtureResult: Future<Output = ()>,
    {
        let kamu_api_server = self.into_kamu_api_server();

        fixture(kamu_api_server).await;
    }

    fn into_kamu_api_server(self) -> KamuNodePuppet {
        let KamuNodeApiServerHarnessOptions {
            env_vars,
            kamu_api_server_config,
            ..
        } = self.options;

        KamuNodePuppet::new_workspace_tmp_with(NewWorkspaceOptions {
            kamu_api_server_config,
            env_vars,
            repo_path: None,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn merge_yaml(
    target: Option<serde_yaml::Value>,
    source: Option<serde_yaml::Value>,
) -> Option<serde_yaml::Value> {
    match (target, source) {
        (Some(mut target), Some(source)) => {
            let target_mapping = target.as_mapping_mut().unwrap();
            let serde_yaml::Value::Mapping(source_mapping) = source else {
                panic!("source is not a mapping: {source:?}")
            };

            target_mapping.extend(source_mapping);

            Some(target)
        }
        (target, None) => target,
        (None, generated_kamu_api_config) => generated_kamu_api_config,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
