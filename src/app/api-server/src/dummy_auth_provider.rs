// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::collections::HashMap;

use internal_error::{InternalError, ResultIntoInternal};
use kamu::domain::auth;
use serde::{Deserialize, Serialize};

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct DummyAuthProvider {
    predefined_accounts: HashMap<String, auth::AccountInfo>,
}

impl DummyAuthProvider {
    pub(crate) fn new_with_default_account() -> Self {
        Self::new(vec![auth::AccountInfo {
            account_id: opendatafabric::FAKE_ACCOUNT_ID.to_string(),
            account_name: opendatafabric::AccountName::new_unchecked(auth::DEFAULT_ACCOUNT_NAME),
            account_type: auth::AccountType::User,
            display_name: String::from(auth::DEFAULT_ACCOUNT_NAME),
            avatar_url: Some(String::from(auth::DEFAULT_AVATAR_URL)),
            is_admin: true,
        }])
    }

    pub(crate) fn new(predefined_accounts: Vec<auth::AccountInfo>) -> Self {
        Self {
            predefined_accounts: predefined_accounts
                .into_iter()
                .map(|acc| (acc.account_name.to_string(), acc))
                .collect(),
        }
    }

    fn find_account_info_impl(&self, account_name: &String) -> Option<auth::AccountInfo> {
        // The account might be predefined in the configuration
        self.predefined_accounts.get(account_name).cloned()
    }

    fn get_account_info_impl(
        &self,
        account_name: &String,
    ) -> Result<auth::AccountInfo, auth::RejectedCredentialsError> {
        // The account might be predefined in the configuration
        match self.predefined_accounts.get(account_name) {
            // Use the predefined record
            Some(account_info) => Ok(account_info.clone()),

            None => {
                // Otherwise we don't recognized this user between predefined
                Err(auth::RejectedCredentialsError::new(
                    "Login of unknown accounts is disabled".to_string(),
                ))
            }
        }
    }
}

#[async_trait::async_trait]
impl auth::AuthenticationProvider for DummyAuthProvider {
    fn login_method(&self) -> &'static str {
        "password"
    }

    async fn login(
        &self,
        login_credentials_json: String,
    ) -> Result<auth::ProviderLoginResponse, auth::ProviderLoginError> {
        // Decode credentials
        let password_login_credentials =
            serde_json::from_str::<PasswordLoginCredentials>(login_credentials_json.as_str())
                .map_err(|e| {
                    auth::ProviderLoginError::InvalidCredentials(
                        auth::InvalidCredentialsError::new(Box::new(e)),
                    )
                })?;

        // For now password should match the login, this is enough for CLI demo needs
        if password_login_credentials.password != password_login_credentials.login {
            return Err(auth::ProviderLoginError::RejectedCredentials(
                auth::RejectedCredentialsError::new("Invalid login or password".into()),
            ));
        }

        // The account might be predefined in the configuration
        let account_info = self
            .get_account_info_impl(&password_login_credentials.login)
            .map_err(auth::ProviderLoginError::RejectedCredentials)?;

        // Store login as provider credentials
        let provider_credentials = PasswordProviderCredentials {
            account_name: account_info.account_name.clone(),
        };

        Ok(auth::ProviderLoginResponse {
            provider_credentials_json: serde_json::to_string::<PasswordProviderCredentials>(
                &provider_credentials,
            )
            .int_err()?,
            account_info,
        })
    }

    async fn account_info_by_token(
        &self,
        provider_credentials_json: String,
    ) -> Result<auth::AccountInfo, InternalError> {
        let provider_credentials =
            serde_json::from_str::<PasswordProviderCredentials>(provider_credentials_json.as_str())
                .int_err()?;

        let account_info = self
            .get_account_info_impl(&provider_credentials.account_name.to_string())
            .int_err()?;

        Ok(account_info)
    }

    async fn find_account_info_by_name<'a>(
        &'a self,
        account_name: &'a opendatafabric::AccountName,
    ) -> Result<Option<auth::AccountInfo>, InternalError> {
        Ok(self.find_account_info_impl(&account_name.into()))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PasswordLoginCredentials {
    pub login: String,
    pub password: String,
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PasswordProviderCredentials {
    pub account_name: opendatafabric::AccountName,
}

///////////////////////////////////////////////////////////////////////////////
