// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use askama::Template;
use dill::{component, interface, meta, Catalog};
use email_gateway::EmailSender;
use internal_error::{InternalError, ResultIntoInternal};
use kamu_accounts::{
    AccountLifecycleMessage,
    AccountLifecycleMessageCreated,
    AccountLifecycleMessageDeleted,
    MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
};
use messaging_outbox::{
    InitialConsumerBoundary,
    MessageConsumer,
    MessageConsumerMeta,
    MessageConsumerT,
    MessageDeliveryMechanism,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(strum_macros::AsRefStr)]
pub enum EmailSubjectAccountLifecycle {
    #[strum(serialize = "Welcome to Kamu!")]
    Created,
    #[strum(serialize = "⚠️ Your account is deleted!")]
    Deleted,
}

pub const MESSAGE_CONSUMER_KAMU_API_SERVER_ACCOUNT_LIFECYCLE_NOTIFIER: &str =
    "dev.kamu.api-server.AccountLifecycleNotifier";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AccountLifecycleNotifier {
    email_sender: Arc<dyn EmailSender>,
}

#[component(pub)]
#[interface(dyn MessageConsumer)]
#[interface(dyn MessageConsumerT<AccountLifecycleMessage>)]
#[meta(MessageConsumerMeta {
    consumer_name: MESSAGE_CONSUMER_KAMU_API_SERVER_ACCOUNT_LIFECYCLE_NOTIFIER,
    feeding_producers: &[
        MESSAGE_PRODUCER_KAMU_ACCOUNTS_SERVICE,
    ],
    delivery: MessageDeliveryMechanism::Transactional,
    initial_consumer_boundary: InitialConsumerBoundary::Latest,
})]
impl AccountLifecycleNotifier {
    pub fn new(email_sender: Arc<dyn EmailSender>) -> Self {
        Self { email_sender }
    }

    async fn notify_account_created(
        &self,
        created: &AccountLifecycleMessageCreated,
    ) -> Result<(), InternalError> {
        let registration_email = RegistrationEmail {
            username: &created.display_name,
        };
        let rendered_registration_body = registration_email.render().unwrap();

        self.email_sender
            .send_email(
                &created.email,
                EmailSubjectAccountLifecycle::Created.as_ref(),
                &rendered_registration_body,
            )
            .await
            .int_err()
    }

    async fn notify_account_deleted(
        &self,
        message: &AccountLifecycleMessageDeleted,
    ) -> Result<(), InternalError> {
        let registration_email = AccountDeletedEmail {
            username: &message.display_name,
        };
        let rendered_registration_body = registration_email.render().unwrap();

        self.email_sender
            .send_email(
                &message.email,
                EmailSubjectAccountLifecycle::Deleted.as_ref(),
                &rendered_registration_body,
            )
            .await
            .int_err()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl MessageConsumer for AccountLifecycleNotifier {}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl MessageConsumerT<AccountLifecycleMessage> for AccountLifecycleNotifier {
    #[tracing::instrument(
        level = "debug",
        skip_all,
        name = "AccountLifecycleNotifier[AccountLifecycleMessage]"
    )]
    async fn consume_message(
        &self,
        _: &Catalog,
        message: &AccountLifecycleMessage,
    ) -> Result<(), InternalError> {
        tracing::debug!(received_message = ?message, "Received account lifecycle message");
        match message {
            AccountLifecycleMessage::Created(created) => self.notify_account_created(created).await,
            AccountLifecycleMessage::Deleted(deleted) => self.notify_account_deleted(deleted).await,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Template)]
#[template(path = "account-registered.html")]
struct RegistrationEmail<'a> {
    username: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Template)]
#[template(path = "account-deleted.html")]
struct AccountDeletedEmail<'a> {
    username: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
