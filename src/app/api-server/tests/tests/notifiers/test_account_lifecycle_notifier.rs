// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use email_gateway::FakeEmailSender;
use email_utils::Email;
use kamu_accounts::{
    AccountDisplayName,
    AccountLifecycleMessage,
    AccountLifecycleMessageDeleted,
    AccountLifecycleMessagePasswordChanged,
    MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
};
use kamu_api_server::{AccountLifecycleNotifier, EmailSubjectAccountLifecycle};
use messaging_outbox::{Outbox, OutboxExt, OutboxImmediateImpl, register_message_dispatcher};
use pretty_assertions::assert_eq;

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
    assert_eq!(1, emails.len());

    let registration_email = emails.first().unwrap();
    assert_eq!("wasya@example.com", registration_email.recipient.as_ref());
    assert_eq!(
        EmailSubjectAccountLifecycle::Created.as_ref(),
        registration_email.subject,
    );
    assert!(
        registration_email.body.contains("Wasya Pupkin"),
        "{}",
        registration_email.body
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_account_deleted_sends_deletion_email() {
    let harness = AccountLifecycleNotifierHarness::new();
    harness
        .send_account_deleted(AccountLifecycleMessageDeleted {
            account_id: odf::AccountID::new_generated_ed25519().1,
            email: Email::parse("wasya@example.com").unwrap(),
            display_name: "Wasya Pupkin".to_string(),
        })
        .await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(1, emails.len());

    let registration_email = emails.first().unwrap();
    assert_eq!("wasya@example.com", registration_email.recipient.as_ref());
    assert_eq!(
        EmailSubjectAccountLifecycle::Deleted.as_ref(),
        registration_email.subject,
    );
    assert!(
        registration_email.body.contains("Wasya Pupkin"),
        "{}",
        registration_email.body
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_password_change_sends_notification_email() {
    let harness = AccountLifecycleNotifierHarness::new();
    harness
        .send_account_password_changed(AccountLifecycleMessagePasswordChanged {
            account_id: odf::AccountID::new_generated_ed25519().1,
            email: Email::parse("wasya@example.com").unwrap(),
            display_name: "Wasya Pupkin".to_string(),
        })
        .await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(1, emails.len());

    let registration_email = emails.first().unwrap();
    assert_eq!("wasya@example.com", registration_email.recipient.as_ref());
    assert_eq!(
        EmailSubjectAccountLifecycle::PasswordChanged.as_ref(),
        registration_email.subject,
    );
    assert!(
        registration_email
            .body
            .contains("your account password has been changed"),
        "{}",
        registration_email.body
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct AccountLifecycleNotifierHarness {
    outbox: Arc<dyn Outbox>,
    fake_email_sender: Arc<FakeEmailSender>,
}

impl AccountLifecycleNotifierHarness {
    fn new() -> Self {
        let mut b = dill::Catalog::builder();

        b.add::<AccountLifecycleNotifier>()
            .add_builder(messaging_outbox::OutboxImmediateImpl::builder(
                messaging_outbox::ConsumerFilter::AllConsumers,
            ))
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

    async fn send_account_deleted(&self, message: AccountLifecycleMessageDeleted) {
        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
                AccountLifecycleMessage::Deleted(message),
            )
            .await
            .unwrap();
    }

    async fn send_account_password_changed(&self, message: AccountLifecycleMessagePasswordChanged) {
        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
                AccountLifecycleMessage::PasswordChanged(message),
            )
            .await
            .unwrap();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
