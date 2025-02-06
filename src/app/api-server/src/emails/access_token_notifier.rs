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
    AccessTokenLifecycleMessage,
    AccessTokenLifecycleMessageCreated,
    MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
};
use messaging_outbox::{
    MessageConsumer,
    MessageConsumerMeta,
    MessageConsumerT,
    MessageDeliveryMechanism,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub const ACCESS_TOKEN_CREATED_SUBJECT: &str = "Access token created";

pub const MESSAGE_CONSUMER_KAMU_API_SERVER_ACCESS_TOKEN_LIFECYCLE_NOTIFIER: &str =
    "dev.kamu.api-server.AccessTokenLifecycleNotifier";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AccessTokenLifecycleNotifier {
    email_sender: Arc<dyn EmailSender>,
    authentication_service: Arc<dyn kamu_accounts::AuthenticationService>,
}

#[component(pub)]
#[interface(dyn MessageConsumer)]
#[interface(dyn MessageConsumerT<AccessTokenLifecycleMessage>)]
#[meta(MessageConsumerMeta {
    consumer_name: MESSAGE_CONSUMER_KAMU_API_SERVER_ACCESS_TOKEN_LIFECYCLE_NOTIFIER,
    feeding_producers: &[
        MESSAGE_PRODUCER_KAMU_ACCESS_TOKEN_SERVICE,
    ],
    delivery: MessageDeliveryMechanism::Transactional,
})]
impl AccessTokenLifecycleNotifier {
    pub fn new(
        email_sender: Arc<dyn EmailSender>,
        authentication_service: Arc<dyn kamu_accounts::AuthenticationService>,
    ) -> Self {
        Self {
            email_sender,
            authentication_service,
        }
    }

    async fn notify_access_token_created(
        &self,
        created_token: &AccessTokenLifecycleMessageCreated,
    ) -> Result<(), InternalError> {
        let registration_email = AccessTokenCreatedEmail {
            token_name: &created_token.token_name,
        };
        let rendered_registration_body = registration_email.render().unwrap();

        let owner_account_res = self
            .authentication_service
            .account_by_id(&created_token.owner_id)
            .await
            .int_err()?;

        let owner_account = owner_account_res.unwrap();

        self.email_sender
            .send_email(
                &owner_account.email,
                ACCESS_TOKEN_CREATED_SUBJECT,
                &rendered_registration_body,
            )
            .await
            .int_err()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl MessageConsumer for AccessTokenLifecycleNotifier {}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl MessageConsumerT<AccessTokenLifecycleMessage> for AccessTokenLifecycleNotifier {
    #[tracing::instrument(
        level = "debug",
        skip_all,
        name = "AccessTokenLifecycleNotifier[AccessTokenLifecycleMessage]"
    )]
    async fn consume_message(
        &self,
        _: &Catalog,
        message: &AccessTokenLifecycleMessage,
    ) -> Result<(), InternalError> {
        tracing::debug!(received_message = ?message, "Received access token lifecycle message");
        match message {
            AccessTokenLifecycleMessage::Created(created) => {
                self.notify_access_token_created(created).await
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Template)]
#[template(path = "access-token-created.html")]
struct AccessTokenCreatedEmail<'a> {
    token_name: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
