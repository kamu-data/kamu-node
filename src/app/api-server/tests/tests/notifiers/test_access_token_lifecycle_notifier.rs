// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use database_common::NoOpDatabasePlugin;
use dill::*;
use email_gateway::FakeEmailSender;
use kamu::domain::{ServerUrlConfig, TenancyConfig};
use kamu_accounts::{
    AccessTokenLifecycleMessage,
    JwtAuthenticationConfig,
    PredefinedAccountsConfig,
    DEFAULT_ACCOUNT_ID,
    DUMMY_EMAIL_ADDRESS,
    MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
};
use kamu_accounts_inmem::{InMemoryAccessTokenRepository, InMemoryAccountRepository};
use kamu_accounts_services::{
    AccessTokenServiceImpl,
    AccountServiceImpl,
    LoginPasswordAuthProvider,
    PredefinedAccountsRegistrator,
};
use kamu_api_server::{AccessTokenLifecycleNotifier, ACCESS_TOKEN_CREATED_SUBJECT};
use kamu_auth_rebac_inmem::InMemoryRebacRepository;
use kamu_auth_rebac_services::{
    DefaultAccountProperties,
    DefaultDatasetProperties,
    RebacServiceImpl,
};
use messaging_outbox::{register_message_dispatcher, Outbox, OutboxExt, OutboxImmediateImpl};
use time_source::SystemTimeSourceDefault;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_access_token_created_email() {
    let harness = AccessTokenLifecycleNotifierHarness::new().await;
    harness
        .send_access_token_created("foo", DEFAULT_ACCOUNT_ID.clone())
        .await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(emails.len(), 1);

    let access_token_created_email = emails.first().unwrap();
    assert_eq!(
        access_token_created_email.recipient.as_ref(),
        DUMMY_EMAIL_ADDRESS.as_ref()
    );
    assert_eq!(
        access_token_created_email.subject,
        format!("{ACCESS_TOKEN_CREATED_SUBJECT}")
    );
    assert!(access_token_created_email.body.contains("foo"));
    assert!(access_token_created_email
        .body
        .contains("href=\"http://platform.example.com/v/settings/access-tokens\""));
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct AccessTokenLifecycleNotifierHarness {
    _catalog: Catalog,
    outbox: Arc<dyn Outbox>,
    fake_email_sender: Arc<FakeEmailSender>,
}

impl AccessTokenLifecycleNotifierHarness {
    async fn new() -> Self {
        let mut b = dill::CatalogBuilder::new();

        b.add::<AccessTokenLifecycleNotifier>()
            .add_value(TenancyConfig::SingleTenant)
            .add_builder(
                messaging_outbox::OutboxImmediateImpl::builder()
                    .with_consumer_filter(messaging_outbox::ConsumerFilter::AllConsumers),
            )
            .bind::<dyn Outbox, OutboxImmediateImpl>()
            .add::<SystemTimeSourceDefault>()
            .add::<InMemoryAccountRepository>()
            .add::<AccountServiceImpl>()
            .add::<AccessTokenServiceImpl>()
            .add::<InMemoryAccessTokenRepository>()
            .add::<PredefinedAccountsRegistrator>()
            .add::<LoginPasswordAuthProvider>()
            .add::<RebacServiceImpl>()
            .add::<InMemoryRebacRepository>()
            .add_value(DefaultAccountProperties::default())
            .add_value(DefaultDatasetProperties::default())
            .add_value(PredefinedAccountsConfig::single_tenant())
            .add_value(JwtAuthenticationConfig::default())
            .add_value(ServerUrlConfig::new_test(None))
            .add::<FakeEmailSender>();

        NoOpDatabasePlugin::init_database_components(&mut b);

        register_message_dispatcher::<AccessTokenLifecycleMessage>(
            &mut b,
            MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
        );

        let catalog = b.build();

        init_on_startup::run_startup_jobs(&catalog).await.unwrap();

        let outbox = catalog.get_one().unwrap();
        let fake_email_sender = catalog.get_one().unwrap();

        Self {
            _catalog: catalog,
            outbox,
            fake_email_sender,
        }
    }

    async fn send_access_token_created(&self, token_name: &str, account_id: odf::AccountID) {
        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
                AccessTokenLifecycleMessage::created(token_name.to_owned(), account_id),
            )
            .await
            .unwrap();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
