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
use secrecy::SecretString;

use crate::config::{DatabaseConfig, DatabaseCredentialSourceConfig, RemoteDatabaseConfig};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn try_build_db_connection_settings(
    raw_db_config: &DatabaseConfig,
) -> Option<DatabaseConnectionSettings> {
    fn convert(c: &RemoteDatabaseConfig, provider: DatabaseProvider) -> DatabaseConnectionSettings {
        DatabaseConnectionSettings::new(
            provider,
            c.database_name.clone(),
            c.host.clone(),
            c.port,
            c.max_connections,
            c.max_lifetime_secs,
            c.acquire_timeout_secs,
        )
    }

    match raw_db_config {
        DatabaseConfig::Sqlite(c) => {
            let path = Path::new(&c.database_path);
            Some(DatabaseConnectionSettings::sqlite_from(path))
        }
        DatabaseConfig::Postgres(config) => Some(convert(config, DatabaseProvider::Postgres)),
        DatabaseConfig::InMemory => None,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn configure_database_components(
    b: &mut CatalogBuilder,
    raw_db_config: &DatabaseConfig,
    db_connection_settings: DatabaseConnectionSettings,
) {
    match db_connection_settings.provider {
        DatabaseProvider::Postgres => {
            PostgresPlugin::init_database_components(b);

            b.add::<kamu_accounts_postgres::PostgresAccountRepository>();
            b.add::<kamu_accounts_postgres::PostgresAccessTokenRepository>();

            b.add::<kamu_datasets_postgres::PostgresDatasetEnvVarRepository>();
            b.add::<kamu_datasets_postgres::PostgresDatasetEntryRepository>();
            b.add::<kamu_datasets_postgres::PostgresDatasetDependencyRepository>();
            b.add::<kamu_datasets_postgres::PostgresDatasetReferenceRepository>();
            b.add::<kamu_datasets_postgres::PostgresDatasetStatisticsRepository>();

            b.add::<kamu_flow_system_postgres::PostgresFlowConfigurationEventStore>();
            b.add::<kamu_flow_system_postgres::PostgresFlowTriggerEventStore>();
            b.add::<kamu_flow_system_postgres::PostgresFlowEventStore>();

            b.add::<kamu_task_system_postgres::PostgresTaskEventStore>();

            b.add::<kamu_messaging_outbox_postgres::PostgresOutboxMessageRepository>();
            b.add::<kamu_messaging_outbox_postgres::PostgresOutboxMessageConsumptionRepository>();

            b.add::<kamu_auth_rebac_postgres::PostgresRebacRepository>();
        }
        DatabaseProvider::Sqlite => {
            SqlitePlugin::init_database_components(b);

            b.add::<kamu_accounts_sqlite::SqliteAccountRepository>();
            b.add::<kamu_accounts_sqlite::SqliteAccessTokenRepository>();

            b.add::<kamu_datasets_sqlite::SqliteDatasetEnvVarRepository>();
            b.add::<kamu_datasets_sqlite::SqliteDatasetEntryRepository>();
            b.add::<kamu_datasets_sqlite::SqliteDatasetDependencyRepository>();
            b.add::<kamu_datasets_sqlite::SqliteDatasetReferenceRepository>();
            b.add::<kamu_datasets_sqlite::SqliteDatasetStatisticsRepository>();

            b.add::<kamu_flow_system_sqlite::SqliteFlowConfigurationEventStore>();
            b.add::<kamu_flow_system_sqlite::SqliteFlowTriggerEventStore>();
            b.add::<kamu_flow_system_sqlite::SqliteFlowEventStore>();

            b.add::<kamu_task_system_sqlite::SqliteTaskSystemEventStore>();

            b.add::<kamu_messaging_outbox_sqlite::SqliteOutboxMessageRepository>();
            b.add::<kamu_messaging_outbox_sqlite::SqliteOutboxMessageConsumptionRepository>();

            b.add::<kamu_auth_rebac_sqlite::SqliteRebacRepository>();
        }
        DatabaseProvider::MySql | DatabaseProvider::MariaDB => {
            panic!(
                "{} database configuration not supported",
                db_connection_settings.provider
            )
        }
    }

    b.add_value(db_connection_settings);

    init_database_password_provider(b, raw_db_config);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn configure_in_memory_components(b: &mut CatalogBuilder) {
    b.add::<kamu_messaging_outbox_inmem::InMemoryOutboxMessageRepository>();
    b.add::<kamu_messaging_outbox_inmem::InMemoryOutboxMessageConsumptionRepository>();

    b.add::<kamu_accounts_inmem::InMemoryAccountRepository>();
    b.add::<kamu_accounts_inmem::InMemoryAccessTokenRepository>();

    b.add::<kamu_flow_system_inmem::InMemoryFlowConfigurationEventStore>();
    b.add::<kamu_flow_system_inmem::InMemoryFlowTriggerEventStore>();
    b.add::<kamu_flow_system_inmem::InMemoryFlowEventStore>();

    b.add::<kamu_task_system_inmem::InMemoryTaskEventStore>();

    b.add::<kamu_datasets_inmem::InMemoryDatasetEnvVarRepository>();
    b.add::<kamu_datasets_inmem::InMemoryDatasetEntryRepository>();
    b.add::<kamu_datasets_inmem::InMemoryDatasetDependencyRepository>();
    b.add::<kamu_datasets_inmem::InMemoryDatasetReferenceRepository>();
    b.add::<kamu_datasets_inmem::InMemoryDatasetStatisticsRepository>();

    b.add::<kamu_auth_rebac_inmem::InMemoryRebacRepository>();

    NoOpDatabasePlugin::init_database_components(b);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn init_database_password_provider(b: &mut CatalogBuilder, raw_db_config: &DatabaseConfig) {
    match raw_db_config {
        DatabaseConfig::InMemory => unreachable!(),
        DatabaseConfig::Sqlite(_) => {
            b.add::<DatabaseNoPasswordProvider>();
        }
        DatabaseConfig::Postgres(config) => match &config.credentials_policy.source {
            DatabaseCredentialSourceConfig::RawPassword(raw_password_config) => {
                b.add_builder(DatabaseFixedPasswordProvider::builder(
                    SecretString::from(raw_password_config.user_name.clone()),
                    SecretString::from(raw_password_config.raw_password.clone()),
                ));
                b.bind::<dyn DatabasePasswordProvider, DatabaseFixedPasswordProvider>();
            }
            DatabaseCredentialSourceConfig::AwsSecret(aws_secret_config) => {
                b.add_builder(DatabaseAwsSecretPasswordProvider::builder(
                    aws_secret_config.secret_name.clone(),
                ));
                b.bind::<dyn DatabasePasswordProvider, DatabaseAwsSecretPasswordProvider>();
            }
            DatabaseCredentialSourceConfig::AwsIamToken(aws_iam_config) => {
                b.add_builder(DatabaseAwsIamTokenProvider::builder(SecretString::from(
                    aws_iam_config.user_name.clone(),
                )));
                b.bind::<dyn DatabasePasswordProvider, DatabaseAwsIamTokenProvider>();
            }
        },
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn connect_database_initially(
    base_catalog: &dill::Catalog,
) -> Result<dill::Catalog, InternalError> {
    let db_connection_settings = base_catalog
        .get_one::<DatabaseConnectionSettings>()
        .unwrap();
    let db_password_provider = base_catalog
        .get_one::<dyn DatabasePasswordProvider>()
        .unwrap();

    let db_credentials = db_password_provider.provide_credentials().await?;

    match db_connection_settings.provider {
        DatabaseProvider::Postgres => PostgresPlugin::catalog_with_connected_pool(
            base_catalog,
            &db_connection_settings,
            db_credentials.as_ref(),
        )
        .int_err(),
        DatabaseProvider::MySql | DatabaseProvider::MariaDB => {
            MySqlPlugin::catalog_with_connected_pool(
                base_catalog,
                &db_connection_settings,
                db_credentials.as_ref(),
            )
            .int_err()
        }
        DatabaseProvider::Sqlite => {
            SqlitePlugin::catalog_with_connected_pool(base_catalog, &db_connection_settings)
                .await
                .int_err()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn spawn_password_refreshing_job(
    db_config: &DatabaseConfig,
    catalog: &dill::Catalog,
) {
    let password_policy_config = match db_config {
        DatabaseConfig::Sqlite(_) | DatabaseConfig::InMemory => None,
        DatabaseConfig::Postgres(config) => Some(config.credentials_policy.clone()),
    };

    if let Some(rotation_frequency_in_minutes) =
        password_policy_config.and_then(|config| config.rotation_frequency_in_minutes)
    {
        let awaiting_duration = std::time::Duration::from_mins(rotation_frequency_in_minutes);

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
