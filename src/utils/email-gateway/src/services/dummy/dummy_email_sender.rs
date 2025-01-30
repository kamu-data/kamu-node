// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use dill::{component, interface};
use email_utils::Email;

use crate::{EmailSender, SendEmailError};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[component(pub)]
#[interface[dyn EmailSender]]
pub struct DummyEmailSender {}

#[async_trait::async_trait]
impl EmailSender for DummyEmailSender {
    #[tracing::instrument(level = "info", skip_all, fields(recipient = recipient.as_ref(), subject))]
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        body: &str,
    ) -> Result<(), SendEmailError> {
        println!(
            "Dummmy Email Sender: recipient = '{}', subject = '{subject}'",
            recipient.as_ref()
        );
        println!("{body}");

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
