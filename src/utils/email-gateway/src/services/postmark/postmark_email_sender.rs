// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use dill::{Singleton, component, interface, scope};
use email_utils::Email;
use internal_error::ResultIntoInternal;
use secrecy::ExposeSecret;

use crate::{EmailSender, PostmarkGatewaySettings, SendEmailError};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const POSTMARK_EMAIL_API_URL: &str = "https://api.postmarkapp.com/email";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct PostmarkEmailSender {
    postmark_settings: Arc<PostmarkGatewaySettings>,
    rest_client: reqwest::Client,
    from_field: String,
}

#[component(pub)]
#[scope(Singleton)] // To reuse `reqwest::Client`
#[interface[dyn EmailSender]]
impl PostmarkEmailSender {
    pub fn new(postmark_settings: Arc<PostmarkGatewaySettings>) -> Self {
        // Cache the "From" field once - it won't change dynamcially
        let from_field = postmark_settings.compose_from_field();

        Self {
            postmark_settings,
            rest_client: reqwest::Client::new(),
            from_field,
        }
    }

    fn should_ignore_recipient(&self, recipient: &Email) -> bool {
        recipient.host() == "example.com"
    }
}

#[async_trait::async_trait]
impl EmailSender for PostmarkEmailSender {
    #[tracing::instrument(level = "info", skip_all, fields(recipient = recipient.as_ref(), subject))]
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        body: &str,
    ) -> Result<(), SendEmailError> {
        if self.should_ignore_recipient(recipient) {
            tracing::debug!(
                recipient = recipient.as_ref(),
                "Sending email to recipient ignored"
            );
            return Ok(());
        }

        let payload = serde_json::json!({
            "From": self.from_field,
            "To": recipient.as_ref(),
            "Subject": subject,
            "HtmlBody": body,
        });
        let response = self
            .rest_client
            .post(POSTMARK_EMAIL_API_URL)
            .header(
                "X-Postmark-Server-Token",
                self.postmark_settings.api_key.expose_secret(),
            )
            .json(&payload)
            .send()
            .await
            .int_err()?;

        tracing::debug!(response = ?response, "Postmark response");

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
