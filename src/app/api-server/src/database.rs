// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::Path;

use database_common::*;
use dill::*;
use internal_error::{InternalError, ResultIntoInternal};
use secrecy::Secret;

use crate::config::{DatabaseConfig, DatabasePasswordSourceConfig, RemoteDatabaseConfig};

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn try_build_db_credentials(
    raw_db_config: &DatabaseConfig,
) -> Option<DatabaseCredentials> {
    fn convert(c: &RemoteDatabaseConfig, provider: DatabaseProvider) -> DatabaseCredentials {
        DatabaseCredentials::new(
            provider,
            c.user.clone(),
            c.database_name.clone(),
            c.host.clone(),
            c.port,
        )
    }

    match raw_db_config {
        DatabaseConfig::Sqlite(c) => {
            let path = Path::new(&c.database_path);
            Some(DatabaseCredentials::sqlite_from(path))
        }
        DatabaseConfig::Postgres(config) => Some(convert(config, DatabaseProvider::Postgres)),
        DatabaseConfig::InMemory => None,
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn configure_database_components(
    b: &mut CatalogBuilder,
    raw_db_config: &DatabaseConfig,
    db_credentials: DatabaseCredentials,
) {
    // TODO: Remove after adding implementation of FlowEventStore for databases
    b.add::<kamu_flow_system_inmem::FlowEventStoreInMem>();

    // TODO: Delete after preparing services for transactional work and replace with
    //       permanent storage options
    b.add::<kamu_flow_system_inmem::FlowConfigurationEventStoreInMem>();
    b.add::<kamu_task_system_inmem::TaskSystemEventStoreInMemory>();

    match db_credentials.provider {
        DatabaseProvider::Postgres => {
            PostgresPlugin::init_database_components(b);

            b.add::<kamu_accounts_postgres::PostgresAccountRepository>();
            b.add::<kamu_accounts_postgres::PostgresAccessTokenRepository>();
        }
        DatabaseProvider::Sqlite => {
            SqlitePlugin::init_database_components(b);

            b.add::<kamu_accounts_sqlite::SqliteAccountRepository>();
            b.add::<kamu_accounts_sqlite::SqliteAccessTokenRepository>();
        }
        DatabaseProvider::MySql | DatabaseProvider::MariaDB => {
            panic!(
                "{} database configuration not supported",
                db_credentials.provider
            )
        }
    }

    b.add_value(db_credentials);

    init_database_password_provider(b, raw_db_config);
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn configure_in_memory_components(b: &mut CatalogBuilder) {
    b.add::<kamu_accounts_inmem::AccountRepositoryInMemory>();
    b.add::<kamu_accounts_inmem::AccessTokenRepositoryInMemory>();
    b.add::<kamu_flow_system_inmem::FlowConfigurationEventStoreInMem>();
    b.add::<kamu_flow_system_inmem::FlowEventStoreInMem>();
    b.add::<kamu_task_system_inmem::TaskSystemEventStoreInMemory>();

    NoOpDatabasePlugin::init_database_components(b);
}

/////////////////////////////////////////////////////////////////////////////////////////

fn init_database_password_provider(b: &mut CatalogBuilder, raw_db_config: &DatabaseConfig) {
    match raw_db_config {
        DatabaseConfig::InMemory => unreachable!(),
        DatabaseConfig::Sqlite(_) => {
            b.add::<DatabaseNoPasswordProvider>();
        }
        DatabaseConfig::Postgres(config) => match &config.password_policy.source {
            DatabasePasswordSourceConfig::RawPassword(raw_password_config) => {
                b.add_builder(
                    DatabaseFixedPasswordProvider::builder()
                        .with_fixed_password(Secret::new(raw_password_config.raw_password.clone())),
                );
                b.bind::<dyn DatabasePasswordProvider, DatabaseFixedPasswordProvider>();
            }
            DatabasePasswordSourceConfig::AwsSecret(aws_secret_config) => {
                b.add_builder(
                    DatabaseAwsSecretPasswordProvider::builder()
                        .with_secret_name(aws_secret_config.secret_name.clone()),
                );
                b.bind::<dyn DatabasePasswordProvider, DatabaseAwsSecretPasswordProvider>();
            }
            DatabasePasswordSourceConfig::AwsIamToken => {
                b.add::<DatabaseAwsIamTokenProvider>();
            }
        },
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn connect_database_initially(
    base_catalog: &dill::Catalog,
) -> Result<dill::Catalog, InternalError> {
    let db_credentials = base_catalog.get_one::<DatabaseCredentials>().unwrap();
    let db_password_provider = base_catalog
        .get_one::<dyn DatabasePasswordProvider>()
        .unwrap();

    let db_password = db_password_provider.provide_password().await?;
    let db_connection_string = db_credentials.connection_string(db_password);

    match db_credentials.provider {
        DatabaseProvider::Postgres => {
            PostgresPlugin::catalog_with_connected_pool(base_catalog, &db_connection_string)
                .int_err()
        }
        DatabaseProvider::MySql | DatabaseProvider::MariaDB => {
            MySqlPlugin::catalog_with_connected_pool(base_catalog, &db_connection_string).int_err()
        }
        DatabaseProvider::Sqlite => {
            SqlitePlugin::catalog_with_connected_pool(base_catalog, &db_connection_string).int_err()
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn spawn_password_refreshing_job(
    db_config: &DatabaseConfig,
    catalog: &dill::Catalog,
) {
    let password_policy_config = match db_config {
        DatabaseConfig::Sqlite(_) | DatabaseConfig::InMemory => None,
        DatabaseConfig::Postgres(config) => Some(config.password_policy.clone()),
    };

    if let Some(rotation_frequency_in_minutes) =
        password_policy_config.and_then(|config| config.rotation_frequency_in_minutes)
    {
        let awaiting_duration = std::time::Duration::from_secs(rotation_frequency_in_minutes);

        let catalog = catalog.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(awaiting_duration).await;

                let password_refresher =
                    catalog.get_one::<dyn DatabasePasswordRefresher>().unwrap();
                password_refresher
                    .refresh_password()
                    .await
                    .expect("Password refreshing failed");
            }
        });
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
