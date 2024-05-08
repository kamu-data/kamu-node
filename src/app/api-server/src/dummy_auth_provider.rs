// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::collections::HashMap;
use std::sync::Arc;

use dill::{component, interface};
use kamu_accounts::*;
use opendatafabric::{AccountID, AccountName};
use serde::{Deserialize, Serialize};

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) const DUMMY_AUTH_PROVIDER_IDENTITY_KEY: &str = "kamu-node";

/////////////////////////////////////////////////////////s////////////////////////////////

pub(crate) struct DummyAuthProvider {
    predefined_accounts: HashMap<String, AccountConfig>,
}

#[component(pub)]
#[interface(dyn AuthenticationProvider)]
impl DummyAuthProvider {
    pub(crate) fn new(config: Arc<PredefinedAccountsConfig>) -> Self {
        Self {
            predefined_accounts: config
                .predefined
                .iter()
                .map(|account| (account.account_name.to_string(), account.clone()))
                .collect(),
        }
    }

    fn get_account(
        &self,
        account_name: &String,
    ) -> Result<AccountConfig, RejectedCredentialsError> {
        match self.predefined_accounts.get(account_name) {
            Some(account_info) => Ok(account_info.clone()),
            None => Err(RejectedCredentialsError {}),
        }
    }
}

#[async_trait::async_trait]
impl AuthenticationProvider for DummyAuthProvider {
    fn provider_name(&self) -> &'static str {
        PROVIDER_PASSWORD
    }

    fn generate_id(&self, account_name: &AccountName) -> AccountID {
        AccountID::new_seeded_ed25519(account_name.as_bytes())
    }

    async fn login(
        &self,
        login_credentials_json: String,
    ) -> Result<ProviderLoginResponse, ProviderLoginError> {
        let password_login_credentials =
            serde_json::from_str::<PasswordLoginCredentials>(login_credentials_json.as_str())
                .map_err(|e| InvalidCredentialsError::new(Box::new(e)))?;

        let account = self.get_account(&password_login_credentials.login)?;

        if password_login_credentials.password != account.get_password() {
            return Err(RejectedCredentialsError {}.into());
        }

        let display_name = account.get_display_name();

        Ok(ProviderLoginResponse {
            account_name: account.account_name,
            email: account.email,
            display_name,
            account_type: account.account_type,
            avatar_url: account.avatar_url,
            provider_identity_key: String::from(DUMMY_AUTH_PROVIDER_IDENTITY_KEY),
        })
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PasswordLoginCredentials {
    pub login: String,
    pub password: String,
}

///////////////////////////////////////////////////////////////////////////////
