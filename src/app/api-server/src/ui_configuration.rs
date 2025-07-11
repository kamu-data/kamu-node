// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu_accounts_services::PasswordPolicyConfig;
use serde::Serialize;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UIConfiguration {
    pub ingest_upload_file_limit_mb: usize,
    pub feature_flags: UIFeatureFlags,
    pub semantic_search_threshold_score: f32,
    pub min_new_password_length: usize,
}

impl Default for UIConfiguration {
    fn default() -> Self {
        Self {
            ingest_upload_file_limit_mb: 50,
            feature_flags: UIFeatureFlags::default(),
            semantic_search_threshold_score: 0.0,
            min_new_password_length: PasswordPolicyConfig::default().min_new_password_length,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UIFeatureFlags {
    pub enable_logout: bool,
    pub enable_scheduling: bool,
    pub enable_dataset_env_vars_management: bool,
    pub enable_terms_of_service: bool,
    pub allow_anonymous: bool,
}

impl Default for UIFeatureFlags {
    fn default() -> Self {
        Self {
            enable_logout: true,
            enable_scheduling: true,
            enable_terms_of_service: true,
            enable_dataset_env_vars_management: false,
            allow_anonymous: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
