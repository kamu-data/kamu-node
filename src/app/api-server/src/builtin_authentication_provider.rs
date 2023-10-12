// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::collections::HashMap;

use dill::component;
use internal_error::{InternalError, ResultIntoInternal};
use serde::{Deserialize, Serialize};

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct BuiltinAuthenticationProvider {
    predefined_accounts: HashMap<String, kamu::domain::auth::AccountInfo>,
}

#[component(pub)]
impl BuiltinAuthenticationProvider {
    pub(crate) fn new() -> Self {
        let kamu_account = kamu::domain::auth::AccountInfo {
            account_id: opendatafabric::FAKE_ACCOUNT_ID.to_string(),
            account_name: opendatafabric::AccountName::new_unchecked(
                kamu::domain::auth::DEFAULT_ACCOUNT_NAME,
            ),
            account_type: kamu::domain::auth::AccountType::User,
            display_name: String::from(kamu::domain::auth::DEFAULT_ACCOUNT_NAME),
            avatar_url: Some(String::from(kamu::domain::auth::DEFAULT_AVATAR_URL)),
        };

        let mut predefined_accounts = HashMap::new();
        predefined_accounts.insert(
            String::from(kamu::domain::auth::DEFAULT_ACCOUNT_NAME),
            kamu_account,
        );

        Self {
            predefined_accounts,
        }
    }

    fn find_account_info_impl(
        &self,
        account_name: &String,
    ) -> Option<kamu::domain::auth::AccountInfo> {
        // The account might be predefined in the configuration
        self.predefined_accounts
            .get(account_name)
            .map(|an| an.clone())
    }

    fn get_account_info_impl(
        &self,
        account_name: &String,
    ) -> Result<kamu::domain::auth::AccountInfo, kamu::domain::auth::RejectedCredentialsError> {
        // The account might be predefined in the configuration
        match self.predefined_accounts.get(account_name) {
            // Use the predefined record
            Some(account_info) => Ok(account_info.clone()),

            None => {
                // Otherwise we don't recognized this user between predefined
                Err(kamu::domain::auth::RejectedCredentialsError::new(
                    "Login of unknown accounts is disabled".to_string(),
                ))
            }
        }
    }
}

#[async_trait::async_trait]
impl kamu::domain::auth::AuthenticationProvider for BuiltinAuthenticationProvider {
    fn login_method(&self) -> &'static str {
        "password"
    }

    async fn login(
        &self,
        login_credentials_json: String,
    ) -> Result<kamu::domain::auth::ProviderLoginResponse, kamu::domain::auth::ProviderLoginError>
    {
        // Decode credentials
        let password_login_credentials =
            serde_json::from_str::<PasswordLoginCredentials>(login_credentials_json.as_str())
                .map_err(|e| {
                    kamu::domain::auth::ProviderLoginError::InvalidCredentials(
                        kamu::domain::auth::InvalidCredentialsError::new(Box::new(e)),
                    )
                })?;

        // For now password should match the login, this is enough for CLI demo needs
        if password_login_credentials
            .password
            .ne(&password_login_credentials.login)
        {
            return Err(kamu::domain::auth::ProviderLoginError::RejectedCredentials(
                kamu::domain::auth::RejectedCredentialsError::new(
                    "Invalid login or password".into(),
                ),
            ));
        }

        // The account might be predefined in the configuration
        let account_info = self
            .get_account_info_impl(&password_login_credentials.login)
            .map_err(|e| kamu::domain::auth::ProviderLoginError::RejectedCredentials(e))?;

        // Store login as provider credentials
        let provider_credentials = PasswordProviderCredentials {
            account_name: account_info.account_name.clone(),
        };

        Ok(kamu::domain::auth::ProviderLoginResponse {
            provider_credentials_json: serde_json::to_string::<PasswordProviderCredentials>(
                &provider_credentials,
            )
            .int_err()?,
            account_info: account_info.into(),
        })
    }

    async fn account_info_by_token(
        &self,
        provider_credentials_json: String,
    ) -> Result<kamu::domain::auth::AccountInfo, InternalError> {
        let provider_credentials = serde_json::from_str::<PasswordProviderCredentials>(
            &provider_credentials_json.as_str(),
        )
        .int_err()?;

        let account_info = self
            .get_account_info_impl(&provider_credentials.account_name.to_string())
            .int_err()?;

        Ok(account_info)
    }

    async fn find_account_info_by_name<'a>(
        &'a self,
        account_name: &'a opendatafabric::AccountName,
    ) -> Result<Option<kamu::domain::auth::AccountInfo>, InternalError> {
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
