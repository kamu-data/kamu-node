// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::{Arc, Mutex};

use dill::*;
use email_utils::Email;

use crate::{EmailSender, SendEmailError};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FakeEmailSender {
    state: Arc<Mutex<State>>,
}

#[component(pub)]
#[interface(dyn EmailSender)]
#[scope(Singleton)]
impl FakeEmailSender {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }

    pub fn get_recorded_emails(&self) -> Vec<FakeEmailMessage> {
        self.state.lock().unwrap().emails.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct State {
    emails: Vec<FakeEmailMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeEmailMessage {
    pub recipient: Email,
    pub subject: String,
    pub body: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl EmailSender for FakeEmailSender {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        body: &str,
    ) -> Result<(), SendEmailError> {
        self.state.lock().unwrap().emails.push(FakeEmailMessage {
            recipient: recipient.clone(),
            subject: subject.to_string(),
            body: body.to_string(),
        });

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
