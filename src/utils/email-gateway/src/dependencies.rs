// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use dill::CatalogBuilder;

use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum EmailConfig {
    Dummy,
    #[cfg(feature = "postmark")]
    Postmark(PostmarkGatewaySettings),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn register_dependencies(catalog_builder: &mut CatalogBuilder, email_config: EmailConfig) {
    match email_config {
        EmailConfig::Dummy => {
            catalog_builder.add::<DummyEmailSender>();
        }
        #[cfg(feature = "postmark")]
        EmailConfig::Postmark(postmark_settings) => {
            catalog_builder.add_value(postmark_settings);
            catalog_builder.add::<PostmarkEmailSender>();
        }
        #[allow(unreachable_patterns)]
        _ => {
            panic!(
                "Email gateway '{email_config:?}' is unavailable, compile with the corresponding \
                 feature enabled",
            );
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
