// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use email_utils::Email;
use secrecy::SecretString;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct PostmarkGatewaySettings {
    pub sender_address: Email,
    pub sender_name: Option<String>,
    pub api_key: SecretString,
}

impl PostmarkGatewaySettings {
    pub fn compose_from_field(&self) -> String {
        match &self.sender_name {
            // "Display Name" <email@example.com>
            Some(sender_name) => format!("\"{}\" {}", sender_name, self.sender_address.as_ref()),
            None => self.sender_address.as_ref().to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
