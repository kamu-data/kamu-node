// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use dill::*;
use email_gateway::FakeEmailSender;
use email_utils::Email;
use kamu_accounts::{
    AccountDisplayName,
    AccountLifecycleMessage,
    MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
};
use kamu_api_server::{AccountLifecycleNotifier, ACCOUNT_REGISTERED_SUBJECT};
use messaging_outbox::{register_message_dispatcher, Outbox, OutboxExt, OutboxImmediateImpl};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_account_created_sends_registration_email() {
    let harness = AccountLifecycleNotifierHarness::new();
    harness
        .send_account_created(
            odf::AccountName::new_unchecked("wasya"),
            AccountDisplayName::from("Wasya Pupkin"),
            Email::parse("wasya@example.com").unwrap(),
        )
        .await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(emails.len(), 1);

    let registration_email = emails.first().unwrap();
    assert_eq!(registration_email.recipient.as_ref(), "wasya@example.com");
    assert_eq!(registration_email.subject, ACCOUNT_REGISTERED_SUBJECT);
    assert!(registration_email.body.contains("Wasya Pupkin"));
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct AccountLifecycleNotifierHarness {
    _catalog: Catalog,
    outbox: Arc<dyn Outbox>,
    fake_email_sender: Arc<FakeEmailSender>,
}

impl AccountLifecycleNotifierHarness {
    fn new() -> Self {
        let mut b = dill::CatalogBuilder::new();

        b.add::<AccountLifecycleNotifier>()
            .add_builder(
                messaging_outbox::OutboxImmediateImpl::builder()
                    .with_consumer_filter(messaging_outbox::ConsumerFilter::AllConsumers),
            )
            .bind::<dyn Outbox, OutboxImmediateImpl>()
            .add::<FakeEmailSender>();

        register_message_dispatcher::<AccountLifecycleMessage>(
            &mut b,
            MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
        );

        let catalog = b.build();
        let outbox = catalog.get_one().unwrap();
        let fake_email_sender = catalog.get_one().unwrap();

        Self {
            _catalog: catalog,
            outbox,
            fake_email_sender,
        }
    }

    async fn send_account_created(
        &self,
        account_name: odf::AccountName,
        display_name: AccountDisplayName,
        email: Email,
    ) {
        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
                AccountLifecycleMessage::created(
                    odf::AccountID::new_seeded_ed25519(account_name.as_bytes()),
                    email,
                    display_name,
                ),
            )
            .await
            .unwrap();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
