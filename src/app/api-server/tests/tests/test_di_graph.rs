// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu::domain::TenancyConfig;
use test_utils::MinioServer;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

macro_rules! test_di_permutations {
    ($test_name: expr, test_groups: $($test_groups:expr)?) => {
        paste::paste! {
            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_st_inmem">]() {
                $test_name(TenancyConfig::SingleTenant, RepositoriesConfig::InMemory).await;
            }

            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_st_sqlite">]() {
                $test_name(TenancyConfig::SingleTenant, RepositoriesConfig::Sqlite).await;
            }

            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_st_postgres">]() {
                $test_name(TenancyConfig::SingleTenant, RepositoriesConfig::Postgres).await;
            }

            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_mt_inmem">]() {
                $test_name(TenancyConfig::MultiTenant, RepositoriesConfig::InMemory).await;
            }

            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_mt_sqlite">]() {
                $test_name(TenancyConfig::MultiTenant, RepositoriesConfig::Sqlite).await;
            }

            #[test_group::group($($test_groups)?)]
            #[test_log::test(tokio::test)]
            pub async fn [<$test_name "_mt_postgres">]() {
                $test_name(TenancyConfig::MultiTenant, RepositoriesConfig::Postgres).await;
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

test_di_permutations!(test_di_graph_validates_local, test_groups:);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

test_di_permutations!(test_di_graph_validates_remote, test_groups: containerized);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
enum RepositoriesConfig {
    InMemory,
    Sqlite,
    Postgres,
}

async fn test_di_graph_validates_local(
    tenancy_config: TenancyConfig,
    repositories_config: RepositoriesConfig,
) {
    let tempdir = tempfile::tempdir().unwrap();

    let config = get_api_server_config(repositories_config);
    let repo_url = url::Url::from_directory_path(tempdir.path()).unwrap();

    let mut catalog_builder =
        kamu_api_server::init_dependencies(config, &repo_url, tenancy_config, tempdir.path(), None)
            .await
            .unwrap();

    add_database_components(&mut catalog_builder, repositories_config);

    // CurrentAccountSubject is inserted by middlewares, but won't be present in
    // the default dependency graph, so we have to add it manually
    catalog_builder.add_value(kamu_accounts::CurrentAccountSubject::new_test());

    catalog_builder.add_value(kamu_adapter_http::AccessToken::new("some-token"));

    // SessionId is assigned by FlightSQL auth middleware
    catalog_builder.add_value(kamu_adapter_flight_sql::SessionId(
        "some-session-id".to_string(),
    ));

    // TODO: We should ensure this test covers parameters requested by commands and
    // types needed for GQL/HTTP adapter that are currently being constructed
    // manually
    let validate_result = catalog_builder.validate();

    assert!(
        validate_result.is_ok(),
        "{}",
        validate_result.err().unwrap()
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn test_di_graph_validates_remote(
    tenancy_config: TenancyConfig,
    repositories_config: RepositoriesConfig,
) {
    let access_key = "AKIAIOSFODNN7EXAMPLE";
    let secret_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

    // TODO: Reconsider setting env vars in test runs
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", access_key);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_key);
    }

    let tmp_repo_dir = tempfile::tempdir().unwrap();
    let bucket = "test-bucket";
    std::fs::create_dir(tmp_repo_dir.path().join(bucket)).unwrap();

    let minio = MinioServer::new(tmp_repo_dir.path(), access_key, secret_key).await;

    use std::str::FromStr;
    let repo_url = url::Url::from_str(&format!(
        "s3+http://{}:{}/{}",
        minio.address, minio.host_port, bucket
    ))
    .unwrap();

    let config = get_api_server_config(repositories_config);

    let mut catalog_builder = kamu_api_server::init_dependencies(
        config,
        &repo_url,
        tenancy_config,
        tmp_repo_dir.path(),
        None,
    )
    .await
    .unwrap();

    add_database_components(&mut catalog_builder, repositories_config);

    // CurrentAccountSubject is inserted by middlewares, but won't be present in
    // the default dependency graph, so we have to add it manually
    catalog_builder.add_value(kamu_accounts::CurrentAccountSubject::new_test());

    catalog_builder.add_value(kamu_adapter_http::AccessToken::new("some-token"));

    // SessionId is assigned by FlightSQL auth middleware
    catalog_builder.add_value(kamu_adapter_flight_sql::SessionId(
        "some-session-id".to_string(),
    ));

    // TODO: We should ensure this test covers parameters requested by commands and
    // types needed for GQL/HTTP adapter that are currently being constructed
    // manually
    let validate_result = catalog_builder.validate();

    assert!(
        validate_result.is_ok(),
        "{}",
        validate_result.err().unwrap()
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Helpers
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_api_server_config(
    repositories_config: RepositoriesConfig,
) -> kamu_api_server::config::ApiServerConfig {
    use kamu_api_server::config::*;

    let mut config = ApiServerConfig::default();

    match repositories_config {
        RepositoriesConfig::InMemory => config.database = DatabaseConfig::InMemory,
        RepositoriesConfig::Sqlite => {
            config.database = DatabaseConfig::Sqlite(SqliteDatabaseConfig {
                database_path: "will-not-be-created.db.sqlite".to_string(),
            })
        }
        RepositoriesConfig::Postgres => {
            let db_config = RemoteDatabaseConfig {
                credentials_policy: DatabaseCredentialsPolicyConfig {
                    source: DatabaseCredentialSourceConfig::RawPassword(
                        RawDatabasePasswordPolicyConfig {
                            user_name: "".to_string(),
                            raw_password: "".to_string(),
                        },
                    ),
                    rotation_frequency_in_minutes: None,
                },
                database_name: "".to_string(),
                host: "".to_string(),
                port: None,
                max_connections: None,
                max_lifetime_secs: None,
                acquire_timeout_secs: None,
            };
            config.database = DatabaseConfig::Postgres(db_config)
        }
    }

    config
}

fn add_database_components(b: &mut dill::CatalogBuilder, repositories_config: RepositoriesConfig) {
    match repositories_config {
        RepositoriesConfig::InMemory => {
            // Nothing to do
        }
        RepositoriesConfig::Sqlite => {
            let pool = database_common::SqlitePoolOptions::default()
                .connect_lazy("sqlite::memory:")
                .unwrap();
            b.add_value(pool.clone());

            let transaction_ref = database_common::TransactionRef::new(pool);
            b.add_value(transaction_ref);
        }
        RepositoriesConfig::Postgres => {
            let pool = database_common::PgPoolOptions::default()
                .connect_lazy("http://example.com")
                .unwrap();
            b.add_value(pool.clone());

            let transaction_ref = database_common::TransactionRef::new(pool);
            b.add_value(transaction_ref);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
